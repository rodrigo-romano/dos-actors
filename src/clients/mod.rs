/*!
# [Actor](crate::Actor)s clients

The module holds the implementation of the different clients that can be assigned to [Actor]s.

# Example

## Logging

A simple logger with a single entry:
```
use dos_actors::prelude::*;
let logging = Logging::<f64>::default();
```
A logger with 2 entries and pre-allocated with 1000 elements:
```
use dos_actors::prelude::*;
let logging = Logging::<f64>::default().n_entry(2).capacity(1_000);
```
## Signals

A constant signal for 100 steps
```
use dos_actors::prelude::*;
let signal = Signals::new(1, 100).signals(Signal::Constant(3.14));
```

A 2 outputs signal made of a constant and a sinusoid for 100 steps
```
use dos_actors::prelude::*;
let signal = Signals::new(2, 100)
               .output_signal(0, Signal::Constant(3.14))
               .output_signal(1, Signal::Sinusoid{
                                        amplitude: 1f64,
                                        sampling_frequency_hz: 1000f64,
                                        frequency_hz: 20f64,
                                        phase_s: 0f64
               });
```
## Rate transitionner

A sample-and-hold rate transition for a named output/input pair passing a [Vec]
```
use dos_actors::prelude::*;
enum MyIO {};
let sampler = Sampler::<Vec<f64>, MyIO>::default();
```

[Actor]: crate::actor
*/

#[cfg(feature = "windloads")]
pub mod windloads;

#[cfg(feature = "fem")]
pub mod fem;

#[cfg(feature = "mount-ctrl")]
pub mod mount;

#[cfg(feature = "m1-ctrl")]
pub mod m1;

#[cfg(feature = "apache-arrow")]
pub mod arrow_client;

#[cfg(feature = "fsm")]
pub mod fsm;

#[cfg(feature = "crseo")]
pub mod ceo;

#[cfg(feature = "lom")]
pub mod lom;

use crate::{
    io::{Data, Read, Write},
    Update,
};
use std::{any::type_name, fmt::Display, sync::Arc};
mod signals;
#[doc(inline)]
pub use signals::{Signal, Signals};

/// Simple data logging
///
/// Accumulates all the inputs in a single [Vec]
#[derive(Debug)]
pub struct Logging<T> {
    data: Vec<T>,
    n_sample: usize,
    n_entry: usize,
}

impl<T> std::ops::Deref for Logging<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Default for Logging<T> {
    fn default() -> Self {
        Self {
            n_entry: 1,
            data: Vec::new(),
            n_sample: 0,
        }
    }
}
impl<T> Logging<T> {
    /// Sets the # of entries to be logged (default: 1)
    pub fn n_entry(self, n_entry: usize) -> Self {
        Self { n_entry, ..self }
    }
    /// Pre-allocates the size of the vector holding the data
    pub fn capacity(self, capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            ..self
        }
    }
}

impl<T> Logging<T> {
    /// Returns the # of time samples
    pub fn len(&self) -> usize {
        self.n_sample / self.n_entry
    }
    /// Returns the sum of the entry sizes
    pub fn n_data(&self) -> usize {
        self.data.len() / self.len()
    }
    /// Checks if the logger is empty
    pub fn is_empty(&self) -> bool {
        self.n_sample == 0
    }
    /// Returns data chunks the size of the entries
    pub fn chunks(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks(self.n_data())
    }
}

impl<T> Display for Logging<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Logging: ({}x{})={}",
            self.n_data(),
            self.len(),
            self.data.len()
        )
    }
}

impl<T> Update for Logging<T> {}
impl<T: Clone, U> Read<Vec<T>, U> for Logging<T> {
    fn read(&mut self, data: Arc<Data<Vec<T>, U>>) {
        log::debug!("receive {} input: {:}", type_name::<U>(), data.len(),);
        self.data.extend((**data).clone());
        self.n_sample += 1;
    }
}

/// Sample-and-hold rate transitionner
#[derive(Debug)]
pub struct Sampler<T, U>(Arc<Data<T, U>>);
impl<T: Default, U> Default for Sampler<T, U> {
    fn default() -> Self {
        Self(Arc::new(Data::new(T::default())))
    }
}
impl<T, U> Update for Sampler<T, U> {}
impl<T, U> Read<T, U> for Sampler<T, U> {
    fn read(&mut self, data: Arc<Data<T, U>>) {
        self.0 = data;
    }
}
impl<T, U> Write<T, U> for Sampler<T, U> {
    fn write(&mut self) -> Option<Arc<Data<T, U>>> {
        Some(self.0.clone())
    }
}
