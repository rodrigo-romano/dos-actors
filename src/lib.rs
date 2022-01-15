//! # GMT Dynamic Optics Simulation Actors
//!
//! The GMT DOS `Actor`s are the building blocks of the GMT DOS integrated model.
//! Each `actor` has 3 properties:
//!  1. **[inputs](Actor::inputs)**
//!  2. **[outputs](Actor::inputs)**
//!  3. **[client](Actor::client)**
//!
//! ## Input/Outputs
//!
//! [inputs](Actor::inputs) is a collection of [io::Input] and
//! [outputs](Actor::inputs) is a collection of [io::Output].
//! An actor must have at least either 1 [io::Input] or 1 [io::Output].
//! A pair of [io::Input]/[io::Output] is linked with a [channel](flume::bounded) where the [io::Input] is the sender
//! and the [io::Output] is the receiver.
//! The same [io::Output] may be linked to several [io::Input]s.
//! [channel](flume::bounded)s are used to synchronize the [Actor]s: [inputs](Actor::inputs) will wait for incoming [outputs](Actor::inputs).
//!
//! There are 2 special [Actor]s:
//!  - **[Initiator]**: with only outputs
//!  - **[Terminator]**: with only inputs
//!
//! Each [Actor] performs the same [task](Actor::task), within an infinite loop, consisting of 3 operations:
//!  1. [collect](Actor::collect) the inputs if any
//!  2. excutes the [client](Actor::client) methods derived from the [Client] trait
//!  3. [distribute](Actor::distribute) the outputs if any
//!
//! The loop exits when one of the following error happens: [ActorError::NoData], [ActorError::DropSend], [ActorError::DropRecv].
//!
//! ### Sampling rates
//!
//! All the [io::Input]s of an [Actor] are collected are the same rate `NI`, and all the [io::Output]s are distributed at the same rate `NO`, however both [inputs](Actor::inputs) and [outputs](Actor::inputs) rates may be different.
//! The [inputs](Actor::inputs) rate `NI` is inherited from the rate `NO` of [outputs](Actor::outputs) that the data is collected from i.e. `(next actor)::NI=(current actor)::NO`.
//!
//! The rates `NI` or `NO` are defined as the ratio between the simulation sampling frequency `[Hz]` and the actor [Actor::inputs] or [Actor::outputs] sampling frequency `[Hz]`, it must be an integer ≥ 1.
//! If `NI>NO`, [outputs](Actor::outputs) are upsampled with a simple sample-and-hold for `NI/NO` samples.
//! If `NO>NI`, [outputs](Actor::outputs) are decimated by a factor `NO/NI`
//!
//! For a 1000Hz simulation sampling frequency, the following table gives some examples of inputs/outputs sampling frequencies and rate:
//!
//! | Inputs `[Hz]` | Ouputs `[Hz]` | NI | NO | Upsampling | Decimation |
//! |--------------:|--------------:|---:|---:|-----------:|-----------:|
//! | 1000          | 1000          |  1 |  1 | -          |  1         |   
//! | 1000          |  100          |  1 | 10 | -          | 10         |   
//! |  100          | 1000          | 10 |  1 | 10         | -          |   
//! |  500          |  100          |  2 | 10 | -          |  5         |   
//! |  100          |  500          | 10 |  2 | 5          |  -         |   
//!
//! ## Client
//!
//! A [client](Actor::client) must be attached to an [Actor]
//! and t must implement the [Client] trait methods:
//!  - [consume](Client::consume) called after [collect](Actor::collect)ing all the [inputs](Actor::inputs)
//!  - [produce](Client::produce) called before [distribute](Actor::distribute)-ing the [outputs](Actor::outputs)
//!  - [update](Client::update) called in between [consume](Client::consume) and [produce](Client::produce)
//!
//! [consume](Client::consume), [produce](Client::produce) and [update](Client::update) have an identity default implementation.

#[derive(thiserror::Error, Debug)]
pub enum ActorError {
    #[error("receiver dropped")]
    DropRecv(#[from] flume::RecvError),
    #[error("sender dropped")]
    DropSend(#[from] flume::SendError<()>),
    #[error("no new data produced")]
    NoData,
    #[error("no inputs defined")]
    NoInputs,
    #[error("no outputs defined")]
    NoOutputs,
    #[error("no client defined")]
    NoClient,
    #[error("senders dropped")]
    Disconnected,
}
pub type Result<R> = std::result::Result<R, ActorError>;

mod actor;
pub mod io;
pub use actor::{Actor, Initiator, Terminator};

/// Client method specifications
pub trait Client<I, O>: std::fmt::Debug
where
    I: Default,
    O: Default,
{
    /// Processes the [Actor] [inputs](Actor::inputs) for the client
    fn consume(&mut self, _data: Vec<&I>) -> &mut Self {
        self
    }
    /// Generates the [outputs](Actor::outputs) from the client
    fn produce(&mut self) -> Option<Vec<O>> {
        Default::default()
    }
    /// Updates the state of the client
    fn update(&mut self) -> &mut Self {
        self
    }
}

pub fn into_arcx<T>(u: T) -> std::sync::Arc<tokio::sync::Mutex<T>> {
    std::sync::Arc::new(tokio::sync::Mutex::new(u))
}

pub fn error_msg(msg: &str, e: &impl std::error::Error) {
    let mut msg = vec![msg.to_string()];
    msg.push(format!("{}", e));
    let mut current = e.source();
    while let Some(cause) = current {
        msg.push(format!("{}", cause));
        current = cause.source();
    }
    println!("{}", msg.join("(after)"))
}
