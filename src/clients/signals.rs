use crate::{
    io::{Data, Write},
    Update,
};
use std::{ops::Add, sync::Arc};

#[cfg(feature = "noise")]
use rand_distr::{Distribution, Normal, NormalError};

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
    /// White noise
    #[cfg(feature = "noise")]
    WhiteNoise(Normal<f64>),
    /// A simphony?
    Composite(Vec<Signal>),
}
#[cfg(feature = "noise")]
impl Signal {
    /// Create a white noise signal with a standard deviation equal to one
    pub fn white_noise() -> Result<Self, NormalError> {
        Ok(Signal::WhiteNoise(Normal::new(0f64, 1f64)?))
    }
    /// Sets white noise standard deviation
    pub fn std_dev(self, sigma: f64) -> Result<Self, NormalError> {
        if let Signal::WhiteNoise(noise) = self {
            Ok(Signal::WhiteNoise(Normal::new(noise.mean(), sigma)?))
        } else {
            Ok(self)
        }
    }
    /// Adds bias to white noise
    pub fn bias(self, bias: f64) -> Result<Self, NormalError> {
        if let Signal::WhiteNoise(noise) = self {
            Ok(Signal::WhiteNoise(Normal::new(bias, noise.std_dev())?))
        } else {
            Ok(self)
        }
    }
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
            #[cfg(feature = "noise")]
            WhiteNoise(noise) => noise.sample(&mut rand::thread_rng()),
            Composite(signals) => signals.iter().map(|signal| signal.get(i)).sum(),
        }
    }
}

/// Signals generator
#[derive(Debug, Default, Clone)]
pub struct Signals {
    size: usize,
    signals: Vec<Signal>,
    pub step: usize,
    pub n_step: usize,
}
impl Signals {
    /// Create `n` null [Signal::Constant]s valid for `n_step` iterations
    pub fn new(n: usize, n_step: usize) -> Self {
        let signals: Vec<_> = vec![Signal::Constant(0f64); n];
        Self {
            size: n,
            signals,
            step: 0,
            n_step,
        }
    }
    /// Sets the same [Signal] for all outputs
    pub fn signals(self, signal: Signal) -> Self {
        let signals = vec![signal.clone(); self.size];
        Self { signals, ..self }
    }
    /// Sets the [Signal] of output #`k`
    pub fn output_signal(self, k: usize, output_signal: Signal) -> Self {
        let mut signals = self.signals;
        signals[k] = output_signal;
        Self { signals, ..self }
    }
}

impl Add for Signal {
    type Output = Signal;

    fn add(self, rhs: Self) -> Self::Output {
        if let Signal::Composite(mut signals) = self {
            signals.push(rhs);
            Signal::Composite(signals)
        } else {
            Signal::Composite(vec![self, rhs])
        }
    }
}

impl Update for Signals {}
impl<U> Write<Vec<f64>, U> for Signals {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, U>>> {
        log::debug!("write {:?}", self.size);
        if self.step < self.n_step {
            let i = self.step;
            let data = self.signals.iter().map(|signal| signal.get(i)).collect();
            self.step += 1;
            Some(Arc::new(Data::new(data)))
        } else {
            None
        }
    }
}
