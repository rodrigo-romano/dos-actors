//! # GMT Dynamic Optics Simulation Actors
//!
//! The GMT DOS `Actor`s are the building blocks of the GMT DOS integrated model.
//! Each `actor` has 2 properties:
//!  1. **[inputs](Actor::inputs)**
//!  2. **[outputs](Actor::inputs)**
//!
//! [inputs](Actor::inputs) is a collection of [io::Input] and
//! [outputs](Actor::inputs) is a collection of [io::Output].
//! An actor must have at least either 1 [io::Input] or 1 [io::Output].
//! A pair of [io::Input]/[io::Output] is linked with a [channel](flume::bounded) where the [io::Input] is the sender
//! and the [io::Output] is the receiver.
//! The same [io::Output] may be linked to several [io::Input]s.
//!
//! There are 2 uniques [Actor]s:
//!  - **[Initiator]**: with only outputs
//!  - **[Terminator]**: with only inputs
//!
//! Each [Actor] performs the same [task](Actor::task), within an infinite loop, consisting of 3 operations:
//!  1. [collect](Actor::collect) the inputs if any
//!  2. [compute](Actor::compute) the outputs if any based on the inputs
//!  3. [distribute](Actor::distribute) the outputs if any
//!
//! The loop exits when one of the following error happens: [ActorError::NoData], [ActorError::DropSend], [ActorError::DropRecv].
//!
//! ## Inputs/Outputs sampling rate
//!
//! All the [io::Input]s of an [Actor] are collected are the same rate `NI`, and all the [io::Output]s are distributed at the same rate `NO`, however both [inputs](Actor::inputs) and [outputs](Actor::inputs) rates may be different.
//! The [inputs](Actor::inputs) rate `NI` is inherited from the rate `NO` of [outputs](Actor::outputs) that the data is collected from i.e. `(next actor)::NI=(current actor)::NO`.
//!
//! The rates `NI` or `NO` are defined as the ratio between the simulation sampling frequency `[Hz]` and the actor [Actor::inputs] or [Actor::outputs] sampling frequency `[Hz]`, it must be an integer ≥ 1.
//! If `NI>NO`, [outputs](Actor::outputs) are upsampled with a simple sample-and-hold for `NI/NO` samples.
//! If `NO>NI`, [outputs](Actor::outputs) are decimated by a factor `NO/NI`
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
//! A client may be attached to an [Actor].
//! If the client exists, it must implement the [Client] trait methods:
//!  - [consume](Client::consume) called after receiving all the [inputs](Actor::inputs)
//!  - [produce](Client::produce) called before sending the [outputs](Actor::outputs)
//!  - [update](Client::update) called in between [consume](Client::consume) and [produce](Client::produce)
//!
//! [consume](Client::consume), [produce](Client::produce) and [update](Client::update) have an identity default implementation.

#[derive(thiserror::Error, Debug)]
pub enum ActorError {
    #[error("Receiver dropped")]
    DropRecv(#[from] flume::RecvError),
    #[error("Sender dropped")]
    DropSend(#[from] flume::SendError<()>),
    #[error("No new data produced")]
    NoData,
    #[error("No inputs defined")]
    NoInputs,
    #[error("No outputs defined")]
    NoOutputs,
    #[error("No client defined")]
    NoClient,
}
pub type Result<R> = std::result::Result<R, ActorError>;

mod actor;
pub mod io;
pub use actor::{Actor, Initiator, Terminator};

pub(crate) type IO<S> = Vec<S>;

/// Client method specifications
pub trait Client<I, O, const NI: usize, const NO: usize>
where
    I: Default,
    O: Default,
{
    /// Processes the [Actor] [inputs](Actor::inputs) for the client
    fn consume(&mut self, _data: &[io::Input<I, NI>]) -> &mut Self {
        self
    }
    /// Generates the [outputs](Actor::outputs) from the client
    fn produce(&self) -> Option<IO<io::Output<O, NO>>> {
        None
    }
    /// Updates the state of the client
    fn update(&mut self) -> &mut Self {
        self
    }
}
impl<I, O, const NI: usize, const NO: usize> Client<I, O, NI, NO> for ()
where
    I: Default,
    O: Default,
{
}
