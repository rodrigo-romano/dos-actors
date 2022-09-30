/*!
# GMT Dynamic Optics Simulation Actors

The GMT DOS [Actor]s are the building blocks of the GMT DOS integrated model.
Each [Actor] has 3 properties:
 1. input objects
 2. output objects
 3. client

## Input/Outputs

input objects are a collection of inputs and
output objects are a collection of outputs.
An actor must have at least either 1 input or 1 output.
A pair of input/output is linked with a [channel](flume::bounded) where the input is the receiver
and the output is the sender.
The same output may be linked to several inputs.
[channel](flume::bounded)s are used to synchronize the [Actor]s.

Each [Actor] performs the same [task](Actor::task), within an infinite loop, consisting of 3 operations:
 1. receiving the inputs if any
 2. updating the client state
 3. sending the outputs if any

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

A client must be assigned to an [Actor]
and the client must implement some of the following traits:
 - [write](crate::io::Write) if the actor has some outputs,
 - [read](crate::io::Read) if the actor has some inputs,
 - [update](Update), this trait must always be implemented (but the default empty implementation is acceptable)

## Model

An integrated model is build as follows:
 1. select and instanciate the [clients]
 2. assign [clients] to [actor]s
 3. add outputs to the [Actor]s and connect them to inputs of other [Actor]s
 4. build a [model]
 5. Check, run and wait for the [Model](crate::model::Model) completion

For more detailed explanations and examples, check the [actor] and [model] modules.

## Features

*/

use async_trait::async_trait;
use io::Assoc;
use std::{
    any::type_name,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};
use tokio::sync::Mutex;
pub use uid_derive::UID;

pub mod actor;
#[cfg(feature = "clients")]
pub mod clients;
pub mod io;
pub mod model;
#[doc(inline)]
pub use actor::{Actor, Initiator, Task, Terminator, Update};
pub use io::UniqueIdentifier;

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
    #[error("{0} has some inputs but inputs rate is zero")]
    SomeInputsZeroRate(String),
    #[error("{0} has no inputs but a positive inputs rate")]
    NoInputsPositiveRate(String),
    #[error("{0} has some outputs but outputs rate is zero")]
    SomeOutputsZeroRate(String),
    #[error("{0} has no outputs but a positive outputs rate")]
    NoOutputsPositiveRate(String),
    #[error("Orphan output in {0} actor")]
    OrphanOutput(String),
}
pub type Result<R> = std::result::Result<R, ActorError>;

/// Assign inputs to actors
pub trait IntoInputs<'a, T, U, CO, const NO: usize, const NI: usize>
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<Data = T>,
    CO: 'static + Update + Send + io::Write<U>,
{
    /// Creates a new input for 'actor' from the last 'Receiver'
    fn into_input<CI, const N: usize>(self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        CI: 'static + Update + Send + io::Read<U>,
        Self: Sized;
    /// Returns an error if there are any unassigned receivers
    ///
    /// Otherwise return the actor with the new output
    fn confirm(self) -> Result<&'a mut Actor<CO, NI, NO>>
    where
        Self: Sized;
}
// Unique hash for a pair of input/output
fn hasio<CO, const NO: usize, const NI: usize>(output_actor: &mut Actor<CO, NI, NO>) -> u64
where
    CO: Update + Send,
{
    let mut hasher = DefaultHasher::new();
    output_actor.who().hash(&mut hasher);
    let output = output_actor
        .outputs
        .as_mut()
        .and_then(|o| o.last_mut())
        .unwrap();
    output
        .who()
        .split("::")
        .last()
        .unwrap()
        .to_owned()
        .hash(&mut hasher);
    let hash = hasher.finish();
    output.set_hash(hash);
    hash
}
impl<'a, T, U, CO, const NO: usize, const NI: usize> IntoInputs<'a, T, U, CO, NO, NI>
    for (
        &'a mut Actor<CO, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<U>>>>,
    )
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<Data = T>,
    CO: 'static + Update + Send + io::Write<U>,
{
    fn into_input<CI, const N: usize>(mut self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        CI: 'static + Update + Send + io::Read<U>,
    {
        if let Some(recv) = self.1.pop() {
            actor.add_input(recv, hasio(self.0))
        }
        self
    }
    fn confirm(self) -> Result<&'a mut Actor<CO, NI, NO>> {
        if self.1.is_empty() {
            Ok(self.0)
        } else {
            Err(ActorError::OrphanOutput(self.0.who()))
        }
    }
}
/// Interface for data logging types
pub trait Entry<U: UniqueIdentifier> {
    /// Adds an entry to the logger
    fn entry(&mut self, size: usize);
}
/// Assign a new entry to a logging actor
#[async_trait]
pub trait IntoLogsN<CI, const N: usize, const NO: usize>
where
    CI: Update + Send,
{
    async fn logn(self, actor: &mut Actor<CI, NO, N>, size: usize) -> Self
    where
        Self: Sized;
}
#[async_trait]
impl<T, U, CI, CO, const N: usize, const NO: usize, const NI: usize> IntoLogsN<CI, N, NO>
    for (
        &mut Actor<CO, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<U>>>>,
    )
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<Data = T>,
    CI: 'static + Update + Send + io::Read<U> + Entry<U>,
    CO: 'static + Update + Send + io::Write<U>,
{
    /// Creates a new logging entry for the output
    async fn logn(mut self, actor: &mut Actor<CI, NO, N>, size: usize) -> Self {
        if let Some(recv) = self.1.pop() {
            (*actor.client.lock().await).entry(size);
            actor.add_input(recv, hasio(self.0))
        }
        self
    }
}
/// Interface for IO data sizes
pub trait Size<U: UniqueIdentifier> {
    fn len(&self) -> usize;
}
/// Assign a new entry to a logging actor
#[async_trait]
pub trait IntoLogs<CI, const N: usize, const NO: usize>
where
    CI: Update + Send,
{
    async fn log(self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        Self: Sized;
}
#[async_trait]
impl<T, U, CI, CO, const N: usize, const NO: usize, const NI: usize> IntoLogs<CI, N, NO>
    for (
        &mut Actor<CO, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<U>>>>,
    )
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<Data = T>,
    CI: 'static + Update + Send + io::Read<U> + Entry<U>,
    CO: 'static + Update + Send + io::Write<U> + Size<U>,
{
    /// Creates a new logging entry for the output
    async fn log(mut self, actor: &mut Actor<CI, NO, N>) -> Self {
        if let Some(recv) = self.1.pop() {
            (*actor.client.lock().await)
                .entry(<CO as Size<U>>::len(&mut *self.0.client.lock().await));
            actor.add_input(recv, hasio(self.0))
        }
        self
    }
}
/// Actor outputs builder
pub struct ActorOutputBuilder {
    capacity: Vec<usize>,
    bootstrap: bool,
}
impl Default for ActorOutputBuilder {
    fn default() -> Self {
        Self {
            capacity: Vec::new(),
            bootstrap: false,
        }
    }
}
impl ActorOutputBuilder {
    /// Creates a new actor output builder multiplexed `n` times
    pub fn new(n: usize) -> Self {
        Self {
            capacity: vec![1; n],
            ..Default::default()
        }
    }
}

type Rx<U> = flume::Receiver<Arc<io::Data<U>>>;

/// Actor add output interface
pub trait AddOuput<'a, C, const NI: usize, const NO: usize>
where
    C: 'static + Update + Send,
{
    /// Sets the channel to unbounded
    fn unbounded(self) -> Self;
    /// Flags the output to be bootstrapped
    fn bootstrap(self) -> Self;
    /// Multiplexes the output `n` times
    fn multiplex(self, n: usize) -> Self;
    /// Builds the new output
    fn build<U>(self) -> (&'a mut Actor<C, NI, NO>, Vec<Rx<U>>)
    where
        C: io::Write<U>,
        U: 'static + UniqueIdentifier + Send + Sync,
        Assoc<U>: Send + Sync;
}
impl<'a, C, const NI: usize, const NO: usize> AddOuput<'a, C, NI, NO>
    for (&'a mut Actor<C, NI, NO>, ActorOutputBuilder)
where
    C: 'static + Update + Send,
{
    fn unbounded(self) -> Self {
        let n = self.1.capacity.len();
        (
            self.0,
            ActorOutputBuilder {
                capacity: vec![usize::MAX; n],
                ..self.1
            },
        )
    }
    fn bootstrap(self) -> Self {
        (
            self.0,
            ActorOutputBuilder {
                bootstrap: true,
                ..self.1
            },
        )
    }
    fn multiplex(self, n: usize) -> Self {
        (
            self.0,
            ActorOutputBuilder {
                capacity: vec![self.1.capacity[0]; n],
                ..self.1
            },
        )
    }
    fn build<U>(self) -> (&'a mut Actor<C, NI, NO>, Vec<Rx<U>>)
    where
        C: 'static + Update + Send + io::Write<U>,
        U: 'static + Send + Sync + UniqueIdentifier,
        Assoc<U>: Send + Sync,
    {
        use io::{Output, S};
        let (actor, builder) = self;
        let mut txs = vec![];
        let mut rxs = vec![];
        for &cap in &builder.capacity {
            let (tx, rx) = if cap == usize::MAX {
                flume::unbounded::<S<U>>()
            } else {
                flume::bounded::<S<U>>(cap)
            };
            txs.push(tx);
            rxs.push(rx);
        }

        let output: Output<C, Assoc<U>, U, NO> = Output::builder(actor.client.clone())
            .bootstrap(builder.bootstrap)
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
    #[cfg(feature = "clients")]
    pub use super::clients::{
        Logging, OneSignal, Sampler, Signal, Signals, Source, Tick, Timer, Void,
    };
    pub use super::{
        model::Model, Actor, AddOuput, ArcMutex, Initiator, IntoInputs, IntoLogs, IntoLogsN, Task,
        Terminator, UniqueIdentifier, UID,
    };
}
