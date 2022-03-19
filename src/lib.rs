/*!
# GMT Dynamic Optics Simulation Actors

The GMT DOS `Actor`s are the building blocks of the GMT DOS integrated model.
Each `actor` has 3 properties:
 1. inputs
 2. outputs
 3. client

## Input/Outputs

inputs are a collection of `InputObject` and
outputs are a collection of `OutputObject`.
An actor must have at least either 1 input or 1 output.
A pair of input/output is linked with a [channel](flume::bounded) where the input is the sender
and the output is the receiver.
The same output may be linked to several inputs.
[channel](flume::bounded)s are used to synchronize the [Actor]s.

Each [Actor] performs the same [task](Actor::run), within an infinite loop, consisting of 3 operations:
 1. [collect](Actor::collect) the inputs if any
 2. call the client [Update](crate::Update) method
 3. [distribute](Actor::distribute) the outputs if any

The loop exits when one of the following error happens: [ActorError::NoData], [ActorError::DropSend], [ActorError::DropRecv].

### Sampling rates

All the inputs of an [Actor] are collected are the same rate `NI`, and all the outputs are distributed at the same rate `NO`, however both inputs and outputs rates may be different.
The inputs rate `NI` is inherited from the rate `NO` of outputs that the data is collected from i.e. `(next actor)::NI=(current actor)::NO`.

The rates `NI` or `NO` are defined as the ratio between the simulation sampling frequency `[Hz]` and the actor inputs or outputs sampling frequency `[Hz]`, it must be an integer ≥ 1.
If `NI>NO`, outputs are upsampled with a simple sample-and-hold for `NI/NO` samples.
If `NO>NI`, outputs are decimated by a factor `NO/NI`

For a 1000Hz simulation sampling frequency, the following table gives some examples of inputs/outputs sampling frequencies and rate:

| Inputs `[Hz]` | Ouputs `[Hz]` | NI | NO | Upsampling | Decimation |
|--------------:|--------------:|---:|---:|-----------:|-----------:|
| 1000          | 1000          |  1 |  1 | -          |  1         |
| 1000          |  100          |  1 | 10 | -          | 10         |
|  100          | 1000          | 10 |  1 | 10         | -          |
|  500          |  100          |  2 | 10 | -          |  5         |
|  100          |  500          | 10 |  2 | 5          |  -         |

## Client

A client must be passed to an [Actor]
and the client must implement some of the following traits:
 - [write](crate::io::Write) if the actor it belongs to has some outputs,
 - [read](crate::io::Read) if the actor it belongs to has some inputs,
 - [update](Update), this trait must always be implemented

## Features

The crates provides a minimal set of default functionalities that can be augmented by selecting appropriate features at compile time:

 - **windloads** : enables the [CFD loads](crate::clients::windloads::CfdLoads) [Actor] client
 - **fem** : enables the GMT [FEM](crate::clients::fem) [Actor] client
 - **mount-ctrl** : enables the GMT mount [controller](crate::clients::mount) [Actor] client
 - **m1-ctrl** : enables the [Actor]]s for the GMT [M1 control system](crate::clients::m1)
 - **apache-arrow** : enables the [Arrow](crate::clients::arrow_client::Arrow) [Actor] for saving data into the [Parquet](https://docs.rs/parquet) data file format
 - **noise** : enables the [rand] and [rand_distr] crates
*/

use std::{any::type_name, sync::Arc};
use tokio::sync::Mutex;

pub mod actor;
pub mod clients;
pub mod io;
#[doc(inline)]
pub use actor::{Actor, AddOutput, AddOutputBuilder, Initiator, Terminator, Update};

#[derive(thiserror::Error, Debug)]
pub enum ActorError {
    #[error("receiver disconnected")]
    DropRecv(#[from] flume::RecvError),
    #[error("sender disconnected")]
    DropSend(#[from] flume::SendError<()>),
    #[error("no new data produced")]
    NoData,
    #[error("no inputs defined")]
    NoInputs,
    #[error("no outputs defined")]
    NoOutputs,
    #[error("no client defined")]
    NoClient,
    #[error("output {0} dropped")]
    Disconnected(String),
}
pub type Result<R> = std::result::Result<R, ActorError>;

/// Assign inputs to actors
pub trait IntoInputs<CI, const N: usize, const NO: usize>
where
    CI: Update + Send,
{
    fn into_input(self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        Self: Sized;
}
impl<T, U, CI, CO, const N: usize, const NO: usize, const NI: usize> IntoInputs<CI, N, NO>
    for (
        &Actor<CO, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<T, U>>>>,
    )
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync,
    CI: 'static + Update + Send + io::Read<T, U>,
    CO: 'static + Update + Send + io::Write<T, U>,
{
    /// Creates a new input for 'actor' from the last 'Receiver'
    fn into_input(mut self, actor: &mut Actor<CI, NO, N>) -> Self {
        if self.1.is_empty() {
            return self;
        }
        actor.add_input(self.1.pop().unwrap());
        self
    }
}

pub trait AddOuputs<'a, C, const NI: usize, const NO: usize>
where
    C: 'static + Update + Send,
{
    fn unbounded(self) -> Self;
    fn bootstrap(self) -> Self;
    fn build<T, U>(
        self,
    ) -> (
        &'a Actor<C, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<T, U>>>>,
    )
    where
        C: io::Write<T, U>,
        T: 'static + Send + Sync,
        U: 'static + Send + Sync;
}
impl<'a, C, const NI: usize, const NO: usize> AddOuputs<'a, C, NI, NO>
    for (&'a mut Actor<C, NI, NO>, AddOutputBuilder)
where
    C: 'static + Update + Send,
{
    fn unbounded(self) -> Self {
        let n = self.1.capacity.len();
        (
            self.0,
            AddOutputBuilder {
                capacity: vec![usize::MAX; n],
                ..self.1
            },
        )
    }
    fn bootstrap(self) -> Self {
        (
            self.0,
            AddOutputBuilder {
                bootstrap: true,
                ..self.1
            },
        )
    }
    fn build<T, U>(
        self,
    ) -> (
        &'a Actor<C, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<T, U>>>>,
    )
    where
        C: 'static + Update + Send + io::Write<T, U>,
        T: 'static + Send + Sync,
        U: 'static + Send + Sync,
    {
        use io::{Output, S};
        let (actor, builder) = self;
        let mut txs = vec![];
        let mut rxs = vec![];
        for &cap in &builder.capacity {
            let (tx, rx) = if cap == usize::MAX {
                flume::unbounded::<S<T, U>>()
            } else {
                flume::bounded::<S<T, U>>(cap)
            };
            txs.push(tx);
            rxs.push(rx);
        }
        let output: Output<C, T, U, NO> = if builder.bootstrap {
            Output::builder(actor.client.clone()).bootstrap()
        } else {
            Output::builder(actor.client.clone())
        }
        .senders(txs)
        .build();

        if let Some(ref mut outputs) = actor.outputs {
            outputs.push(Box::new(output));
        } else {
            actor.outputs = Some(vec![Box::new(output)]);
        }
        (actor, rxs)
    }
}

/// Creates a reference counted pointer
///
/// Converts an object into an atomic (i.e. thread-safe) reference counted pointer [Arc](std::sync::Arc) with interior mutability [Mutex](tokio::sync::Mutex)
pub trait ArcMutex {
    fn into_arcx(self) -> Arc<Mutex<Self>>
    where
        Self: Sized,
    {
        Arc::new(Mutex::new(self))
    }
}
impl<C: Update> ArcMutex for C {}

pub trait Who<T> {
    /// Returns type name
    fn who(&self) -> String {
        type_name::<T>().to_string()
    }
}

/// Pretty prints error message
pub fn print_error<S: Into<String>>(msg: S, e: &impl std::error::Error) {
    let mut msg: Vec<String> = vec![msg.into()];
    msg.push(format!("{}", e));
    let mut current = e.source();
    while let Some(cause) = current {
        msg.push(format!("{}", cause));
        current = cause.source();
    }
    //println!("{}", msg.join("\n .after: "))
    log::info!("{}", msg.join("\n .after: "))
}

/// Macros to reduce boilerplate code
pub mod macros;

pub mod prelude {
    #[allow(unused_imports)]
    pub use super::{
        actor::Run,
        channel,
        clients::{Logging, Sampler, Signal, Signals},
        run, spawn, spawn_bootstrap, Actor, AddOuputs, AddOutput, ArcMutex, Initiator, IntoInputs,
        Terminator, Who,
    };
}
