/*!
# GMT Dynamic Optics Simulation Actors

The GMT DOS `Actor`s are the building blocks of the GMT DOS integrated model.
Each `actor` has 2 properties:
 1. **[inputs](Actor::inputs)**
 2. **[outputs](Actor::inputs)**

## Input/Outputs

[inputs](Actor::inputs) is a collection of [io::Input] and
[outputs](Actor::inputs) is a collection of [io::Output].
An actor must have at least either 1 [io::Input] or 1 [io::Output].
A pair of [io::Input]/[io::Output] is linked with a [channel](flume::bounded) where the [io::Input] is the sender
and the [io::Output] is the receiver.
The same [io::Output] may be linked to several [io::Input]s.
[channel](flume::bounded)s are used to synchronize the [Actor]s: [inputs](Actor::inputs) will wait for incoming [outputs](Actor::inputs).

There are 2 special [Actor]s:
 - **[Initiator]**: with only outputs
 - **[Terminator]**: with only inputs

Each [Actor] performs the same [task](Actor::run), within an infinite loop, consisting of 3 operations:
 1. [collect](Actor::collect) the inputs if any
 2. excutes the client methods derived from the [Client] trait
 3. [distribute](Actor::distribute) the outputs if any

The loop exits when one of the following error happens: [ActorError::NoData], [ActorError::DropSend], [ActorError::DropRecv].

### Sampling rates

All the [io::Input]s of an [Actor] are collected are the same rate `NI`, and all the [io::Output]s are distributed at the same rate `NO`, however both [inputs](Actor::inputs) and [outputs](Actor::inputs) rates may be different.
The [inputs](Actor::inputs) rate `NI` is inherited from the rate `NO` of [outputs](Actor::outputs) that the data is collected from i.e. `(next actor)::NI=(current actor)::NO`.

The rates `NI` or `NO` are defined as the ratio between the simulation sampling frequency `[Hz]` and the actor [Actor::inputs] or [Actor::outputs] sampling frequency `[Hz]`, it must be an integer ≥ 1.
If `NI>NO`, [outputs](Actor::outputs) are upsampled with a simple sample-and-hold for `NI/NO` samples.
If `NO>NI`, [outputs](Actor::outputs) are decimated by a factor `NO/NI`

For a 1000Hz simulation sampling frequency, the following table gives some examples of inputs/outputs sampling frequencies and rate:

| Inputs `[Hz]` | Ouputs `[Hz]` | NI | NO | Upsampling | Decimation |
|--------------:|--------------:|---:|---:|-----------:|-----------:|
| 1000          | 1000          |  1 |  1 | -          |  1         |
| 1000          |  100          |  1 | 10 | -          | 10         |
|  100          | 1000          | 10 |  1 | 10         | -          |
|  500          |  100          |  2 | 10 | -          |  5         |
|  100          |  500          | 10 |  2 | 5          |  -         |

## Client

A client must be passed to an [Actor] [task](Actor::run)
and the client must implement the [Client] trait methods:
 - [consume](Client::consume) called after [collect](Actor::collect)ing all the [inputs](Actor::inputs)
 - [produce](Client::produce) called before [distribute](Actor::distribute)-ing the [outputs](Actor::outputs)
 - [update](Client::update) called before [produce](Client::produce)

[consume](Client::consume), [produce](Client::produce) and [update](Client::update) have an identity default implementation.

## Features

The crates provides a minimal set of default functionalities that can be augmented by selecting appropriate features at compile time:

 - **windloads** : enables the [CFD loads](crate::clients::windloads::CfdLoads) [Actor] [Client]
 - **fem** : enables the GMT [FEM](crate::clients::fem) [Actor] [Client]
 - **mount-ctrl** : enables the GMT mount [controller](crate::clients::mount::mount_ctrlr) and [driver](crate::clients::mount::mount_drives) [Actor] [Client]s
 - **m1-ctrl** : enables the [Actor] [Client]s for the GMT [M1 control system](crate::clients::m1)
 - **apache-arrow** : enables the [Arrow](crate::clients::arrow_client::Arrow) [Actor] [Client] for saving data into the [Parquet](https://docs.rs/parquet) data file format
 - **noise** : enables the [rand] and [rand_distr] crates
*/

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
    #[error("outputs disconnected")]
    Disconnected,
}
pub type Result<R> = std::result::Result<R, ActorError>;

mod actor;
pub mod io;
use std::{any::type_name, sync::Arc};

pub use actor::{Actor, Initiator, Terminator, Update};

pub mod clients;
#[doc(inline)]
pub use clients::Client;
use tokio::sync::Mutex;

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
    println!("{}", msg.join("\n .after: "))
}

/// Macros to reduce boilerplate code
pub mod macros;

pub mod prelude {
    #[allow(unused_imports)]
    pub use super::{
        channel,
        clients::{Logging, Sampler, Signal, Signals},
        count, run, spawn, spawn_bootstrap, stage, Actor, ArcMutex, Client, Initiator, IntoInputs,
        Terminator, Who,
    };
}
