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
//!  - [update](Client::update) called before [produce](Client::produce)
//!
//! [consume](Client::consume), [produce](Client::produce) and [update](Client::update) have an identity default implementation.
//!
//! ## Example
//!
//! ```
//! use dos_actors::{Actor, Client, Initiator, Terminator};
//! use rand_distr::{Distribution, Normal};
//! use std::{ops::Deref, time::Instant};
//!
//! #[derive(Default, Debug)]
//! struct Signal {
//!     pub sampling_frequency: f64,
//!     pub period: f64,
//!     pub n_step: usize,
//!     pub step: usize,
//! }
//! impl Client for Signal {
//!     type I = ();
//!     type O = f64;
//!     fn produce(&mut self) -> Option<Vec<Self::O>> {
//!         if self.step < self.n_step {
//!             let value = (2.
//!                 * std::f64::consts::PI
//!                 * self.step as f64
//!                 * (self.sampling_frequency * self.period).recip())
//!             .sin()
//!                 - 0.25
//!                     * (2.
//!                         * std::f64::consts::PI
//!                         * ((self.step as f64
//!                             * (self.sampling_frequency * self.period * 0.25).recip())
//!                             + 0.1))
//!                         .sin();
//!             self.step += 1;
//!             Some(vec![value, value])
//!         } else {
//!             None
//!         }
//!     }
//! }
//! #[derive(Default, Debug)]
//! struct Logging(Vec<f64>);
//! impl Deref for Logging {
//!     type Target = Vec<f64>;
//!     fn deref(&self) -> &Self::Target {
//!         &self.0
//!     }
//! }
//! impl Client for Logging {
//!     type I = f64;
//!     type O = ();
//!     fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
//!         self.0.extend(data.into_iter());
//!         self
//!     }
//! }
//!
//! #[derive(Debug)]
//! struct Filter {
//!     data: f64,
//!     noise: Normal<f64>,
//!     step: usize,
//! }
//! impl Default for Filter {
//!     fn default() -> Self {
//!         Self {
//!             data: 0f64,
//!             noise: Normal::new(0.3, 0.05).unwrap(),
//!             step: 0,
//!         }
//!     }
//! }
//! impl Client for Filter {
//!     type I = f64;
//!     type O = f64;
//!     fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
//!         self.data = *data[0];
//!         self
//!     }
//!     fn update(&mut self) -> &mut Self {
//!         self.data += 0.05
//!             * (2. * std::f64::consts::PI * self.step as f64 * (1e3f64 * 2e-2).recip()).sin()
//!             + self.noise.sample(&mut rand::thread_rng());
//!         self.step += 1;
//!         self
//!     }
//!     fn produce(&mut self) -> Option<Vec<Self::O>> {
//!         Some(vec![self.data])
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let n_sample = 2001;
//!     let sim_sampling_frequency = 1000f64;
//!
//!     let mut signal = Signal {
//!         sampling_frequency: sim_sampling_frequency,
//!         period: 1f64,
//!         n_step: n_sample,
//!         step: 0,
//!     };
//!     let mut logging = Logging::default();
//!
//!     let mut source = Initiator::<f64, 1>::build();
//!     let mut filter = Actor::<f64, f64, 1, 1>::new();
//!     let mut sink = Terminator::<f64, 1>::build();
//!
//!     dos_actors::channel(&mut source, &mut [&mut filter]);
//!     dos_actors::channel(&mut filter, &mut [&mut sink]);
//!     dos_actors::channel(&mut source, &mut [&mut sink]);
//!
//!     tokio::spawn(async move {
//!         if let Err(e) = source.run(&mut signal).await {
//!             dos_actors::print_error("Source loop ended", &e);
//!         }
//!     });
//!     tokio::spawn(async move {
//!         if let Err(e) = filter.run(&mut Filter::default()).await {
//!             dos_actors::print_error("Filter loop ended", &e);
//!         }
//!     });
//!     let now = Instant::now();
//!     if let Err(e) = sink.run(&mut logging).await {
//!         dos_actors::print_error("Sink loop ended", &e);
//!     }
//!     println!("Model run in {}ms", now.elapsed().as_millis());
//!
//!     let _: complot::Plot = (
//!         logging
//!             .deref()
//!             .chunks(2)
//!             .enumerate()
//!             .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x.to_vec())),
//!         None,
//!     )
//!         .into();
//!
//!     Ok(())
//! }
//! ```

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

/// Creates a reference counted pointer
///
/// Converts an object into an atomic (i.e. thread-safe) reference counted pointer [Arc](std::sync::Arc) with interior mutability [Mutex](tokio::sync::Mutex)
pub fn into_arcx<T>(object: T) -> std::sync::Arc<tokio::sync::Mutex<T>> {
    std::sync::Arc::new(tokio::sync::Mutex::new(object))
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

/// Macros to reduce boiler plate code
pub mod macros {
    #[macro_export]
    /// Creates input/output channels between pairs of actors
    ///
    /// # Examples
    /// Creates a single channel
    /// ```
    /// channel!(actor1 => actor2)
    /// ```
    /// Creates three channels for the pairs (actor1,actor2), (actor2,actor3) and (actor3,actor4)
    /// ```
    /// channel!(actor1 => actor2  => actor3  => actor4)
    /// ```
    macro_rules! channel {
    () => {};
    ($from:expr => $to:expr) => {
        dos_actors::channel(&mut $from, &mut [&mut $to]);
    };
    ($from:expr => $to:expr $(=> $tail:expr)*) => {
        dos_actors::channel(&mut $from, &mut [&mut $to]);
	channel!($to $(=> $tail)*)
    };
}
    #[macro_export]
    /// Starts an actor loop with an associated client
    ///
    /// # Examples
    /// ```
    /// run!(actor, client)
    /// ```
    macro_rules! run {
        ($actor:expr,$client:expr) => {
            if let Err(e) = $actor.run(&mut $client).await {
                dos_actors::print_error(format!("{} loop ended", stringify!($actor)), &e);
            };
        };
    }
    #[macro_export]
    /// Spawns actors loop with associated clients
    ///
    /// Initial output data may be given, the data will be sent before starting the loop
    ///
    /// # Example
    /// ```
    /// spawn!((actor1, client1,), (actor2, client2,), (actor2, client2, data0))
    /// ```
    macro_rules! spawn {
    ($(($actor:expr,$client:expr,$($init:expr)?)),+) => {
	$(
        tokio::spawn(async move {
	   $(
               if let Err(e) = $actor.distribute(Some($init)).await {
		   dos_actors::print_error(format!("{} distribute ended", stringify!($actor)), &e);
               }
	   )?
		run!($actor,$client);
        });)+
    };
}
}

pub mod prelude {
    #[allow(unused_imports)]
    pub use super::{channel, run, spawn, Actor, Client, Initiator, Terminator};
}
