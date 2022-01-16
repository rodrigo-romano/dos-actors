//! # GMT Dynamic Optics Simulation Actors
//!
//! The GMT DOS `Actor`s are the building blocks of the GMT DOS integrated model.
//! Each `actor` has 2 properties:
//!  1. **[inputs](Actor::inputs)**
//!  2. **[outputs](Actor::inputs)**
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
//! Each [Actor] performs the same [task](Actor::run), within an infinite loop, consisting of 3 operations:
//!  1. [collect](Actor::collect) the inputs if any
//!  2. excutes the client methods derived from the [Client] trait
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
//! A client must be passed to an [Actor] [task](Actor::run)
//! and the client must implement the [Client] trait methods:
//!  - [consume](Client::consume) called after [collect](Actor::collect)ing all the [inputs](Actor::inputs)
//!  - [produce](Client::produce) called before [distribute](Actor::distribute)-ing the [outputs](Actor::outputs)
//!  - [update](Client::update) called following called to [consume](Client::consume)
//!
//! [consume](Client::consume), [produce](Client::produce) and [update](Client::update) have an identity default implementation.

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
    #[error("senders disconnected")]
    Disconnected,
}
pub type Result<R> = std::result::Result<R, ActorError>;

mod actor;
pub mod io;
pub use actor::{Actor, Initiator, Terminator};

/// Client method specifications
pub trait Client: std::fmt::Debug {
    type I;
    type O;
    /// Processes the [Actor] [inputs](Actor::inputs) for the client
    fn consume(&mut self, _data: Vec<&Self::I>) -> &mut Self {
        self
    }
    /// Generates the [outputs](Actor::outputs) from the client
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        Default::default()
    }
    /// Updates the state of the client
    fn update(&mut self) -> &mut Self {
        self
    }
}

/// Add [io::Input]/[io::Output] to [Actor]
pub trait AddIO<I, O, const NI: usize, const NO: usize>
where
    I: Default,
    O: Default,
{
    /// Adds an input to [Actor]
    fn add_input(&mut self, input: io::Input<I, NI>) -> &mut Self;
    /// Adds an output to [Actor]
    fn add_output(&mut self, output: io::Output<O, NO>) -> &mut Self;
}
impl<I, O, const NI: usize, const NO: usize> AddIO<I, O, NI, NO> for Actor<I, O, NI, NO>
where
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    /// Adds an input to [Actor]
    fn add_input(&mut self, input: io::Input<I, NI>) -> &mut Self {
        if let Some(inputs) = self.inputs.as_mut() {
            inputs.push(input);
        } else {
            self.inputs = Some(vec![input]);
        }
        self
    }
    /// Adds an output to [Actor]
    fn add_output(&mut self, output: io::Output<O, NO>) -> &mut Self {
        if let Some(outputs) = self.outputs.as_mut() {
            outputs.push(output);
        } else {
            self.outputs = Some(vec![output]);
        }
        self
    }
}

/// Creates a new channel between 1 sending [Actor] to multiple receiving [Actor]s
pub fn channel<I, T, O, const NI: usize, const N: usize, const NO: usize>(
    sender: &mut impl AddIO<I, T, NI, N>,
    receivers: &mut [&mut impl AddIO<T, O, N, NO>],
) where
    I: Default + std::fmt::Debug,
    T: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    let (output, inputs) = io::channels(receivers.len());
    sender.add_output(output);
    receivers
        .iter_mut()
        .zip(inputs.into_iter())
        .for_each(|(receiver, input)| {
            receiver.add_input(input);
        });
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
    println!("{}", msg.join("(after)"))
}
