//! [Actor](crate::Actor)s [Client]s
//!
//! The module holds the trait [Client] which methods are called
//! by the [Actor](crate::Actor)s client that is passed to the
//! [Actor::run](crate::Actor::run) method
//!
//! A few clients are defined:
//!  - [Logging] that accumulates the data received by a [Terminator](crate::Terminator)
//! into a [Vec]tor
//!  - [Sampler] that moves the data unmodified from inputs to outputs, useful for rate transitions.
//!  - [Signals] that generates some predefined signals

#[cfg(feature = "windloads")]
pub mod windloads;

#[cfg(feature = "fem")]
pub mod fem;

#[cfg(feature = "mount-ctrl")]
pub mod mount;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("cannot open a parquet file")]
    ArrowToFile(#[from] std::io::Error),
    #[cfg(feature = "apache-arrow")]
    #[error("cannot build Arrow data")]
    ArrowError(#[from] arrow::error::ArrowError),
    #[cfg(feature = "apache-arrow")]
    #[error("cannot save data to Parquet")]
    ParquetError(#[from] parquet::errors::ParquetError),
}

/// Client method specifications
pub trait Client {
    //: std::fmt::Debug {
    type I;
    type O;
    /// Processes the [Actor](crate::Actor) [inputs](crate::Actor::inputs) for the client
    fn consume(&mut self, _data: Vec<&Self::I>) -> &mut Self {
        self
    }
    /// Generates the [outputs](crate::Actor::outputs) from the client
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        Default::default()
    }
    /// Updates the state of the client
    fn update(&mut self) -> &mut Self {
        self
    }
}

/// Simple data logging
///
/// Accumulates all the inputs in a single [Vec]
#[derive(Default, Debug)]
pub struct Logging<T>(Vec<T>);
impl<T> std::ops::Deref for Logging<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> Client for Logging<Vec<T>>
where
    T: std::fmt::Debug + Clone,
{
    type I = Vec<T>;
    type O = ();
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        log::debug!(
            "receive #{} inputs: {:?}",
            data.len(),
            data.iter().map(|x| x.len()).collect::<Vec<usize>>()
        );
        self.0.push(data.into_iter().flatten().cloned().collect());
        self
    }
}

#[cfg(feature = "apache-arrow")]
pub mod arrow_client;

/// Sample-and-hold rate transionner
#[derive(Debug, Default)]
pub struct Sampler<T>(Vec<T>);
impl<T> Client for Sampler<T>
where
    T: std::fmt::Debug + Clone,
{
    type I = T;
    type O = T;
    fn consume(&mut self, data: Vec<&Self::I>) -> &mut Self {
        self.0 = data.into_iter().cloned().collect();
        self
    }
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        Some(self.0.drain(..).collect())
    }
}

/// Signal types
#[derive(Debug, Clone)]
pub enum Signal {
    /// A constant signal
    Constant(f64),
    /// A sinusoidal signal
    Sinusoid {
        amplitude: f64,
        sampling_frequency_hz: f64,
        frequency_hz: f64,
        phase_s: f64,
    },
}
impl Signal {
    /// Returns the signal value at step `i`
    pub fn get(&self, i: usize) -> f64 {
        use Signal::*;
        match self {
            Constant(val) => *val,
            Sinusoid {
                amplitude,
                sampling_frequency_hz,
                frequency_hz,
                phase_s,
            } => {
                (2f64
                    * std::f64::consts::PI
                    * (phase_s + i as f64 * frequency_hz / sampling_frequency_hz))
                    .sin()
                    * amplitude
            }
        }
    }
}

/// Signals generator
#[derive(Debug, Default)]
pub struct Signals {
    outputs_size: Vec<usize>,
    signals: Vec<Vec<Signal>>,
    pub step: usize,
    pub n_step: usize,
}
impl Signals {
    /// Create new signals
    pub fn new(outputs_size: Vec<usize>, n_step: usize) -> Self {
        let signal: Vec<_> = outputs_size
            .iter()
            .map(|&n| vec![Signal::Constant(0f64); n])
            .collect();
        Self {
            outputs_size,
            signals: signal,
            step: 0,
            n_step,
        }
    }
    /// Sets the type of signals
    pub fn signals(self, signal: Signal) -> Self {
        let signal: Vec<_> = self
            .outputs_size
            .iter()
            .map(|&n| vec![signal.clone(); n])
            .collect();
        Self {
            signals: signal,
            ..self
        }
    }
    /// Sets the type of signals of one output
    pub fn output_signals(self, output: usize, output_signals: Signal) -> Self {
        let mut signals = self.signals;
        signals[output] = vec![output_signals; self.outputs_size[output]];
        Self { signals, ..self }
    }
    /// Sets the type of signals of one output index
    pub fn output_signal(self, output: usize, output_i: usize, signal: Signal) -> Self {
        let mut signals = self.signals;
        signals[output][output_i] = signal;
        Self { signals, ..self }
    }
}

impl Client for Signals {
    type I = ();
    type O = Vec<f64>;
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        if self.step < self.n_step {
            let i = self.step;
            self.step += 1;
            Some(
                self.signals
                    .iter()
                    .map(|signals| {
                        signals
                            .iter()
                            .map(|signal| signal.get(i))
                            .collect::<Vec<_>>()
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
}
