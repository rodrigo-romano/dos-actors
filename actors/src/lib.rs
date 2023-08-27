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
 4. build a [mod@model]
 5. Check, run and wait for the [Model](crate::model::Model) completion

For more detailed explanations and examples, check the [actor] and [mod@model] modules.

## Features

*/

use std::sync::Arc;
use tokio::sync::Mutex;

pub use gmt_dos_actors_dsl::actorscript;

pub mod actor;
// #[cfg(feature = "clients")]
// pub mod clients;
pub mod io;
pub mod model;
#[doc(inline)]
pub use actor::{Actor, Initiator, Task, Terminator};
mod network;
pub(crate) use gmt_dos_clients::interface::{
    self, print_info, Assoc, Data, Read, UniqueIdentifier, Update, Who,
};
pub(crate) use network::ActorOutputBuilder;
pub use network::{AddOuput, IntoInputs, IntoLogs, IntoLogsN, TryIntoInputs};

#[derive(thiserror::Error, Debug)]
pub enum ActorError {
    #[error("{msg} receiver disconnected")]
    DropRecv {
        msg: String,
        source: flume::RecvError,
    },
    #[error("{msg} sender disconnected")]
    DropSend {
        msg: String,
        source: flume::SendError<()>,
    },
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
    #[error("{0} has no inputs but a positive inputs rate (May be this Actor should instead be an Initiator)")]
    NoInputsPositiveRate(String),
    #[error("{0} has some outputs but outputs rate is zero")]
    SomeOutputsZeroRate(String),
    #[error("{0} has no outputs but a positive outputs rate (May be this Actor should instead be a Terminator)")]
    NoOutputsPositiveRate(String),
    #[error(r#"Orphan output "{0}" in "{1}" actor"#)]
    OrphanOutput(String, String),
}
pub type Result<R> = std::result::Result<R, ActorError>;

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

/// Macros to reduce boilerplate code
pub mod macros;

pub mod prelude {
    pub use super::{
        model, model::Model, Actor, AddOuput, ArcMutex, Initiator, IntoInputs, IntoLogs, IntoLogsN,
        Task, Terminator, TryIntoInputs,
    };
    pub use vec_box::vec_box;
}
