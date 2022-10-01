/*!
# [Actor](crate::Actor)s clients

The module holds the implementation of the different clients that can be assigned to [Actor]s.

Any structure can become a client to an Actor if it implements the [Update] trait with either or both [Read] and [Write] traits.

# Example

## Logging

A simple logger with a single entry:
```
use gmt_dos_actors::prelude::*;
let logging = Logging::<f64>::default();
```
A logger with 2 entries and pre-allocated with 1000 elements:
```
use gmt_dos_actors::prelude::*;
let logging = Logging::<f64>::default().n_entry(2).capacity(1_000);
```
## Signals

A constant signal for 100 steps
```
use gmt_dos_actors::prelude::*;
let signal = Signals::new(1, 100).signals(Signal::Constant(3.14));
```

A 2 outputs signal made of a constant and a sinusoid for 100 steps
```
use gmt_dos_actors::prelude::*;
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

A rate transition actor for a named output/input pair sampling a [Vec]
```
use gmt_dos_actors::prelude::*;
#[derive(UID)]
enum MyIO {};
let sampler = Sampler::<Vec<f64>, MyIO>::default();
```

## Alias to input/output UID

Creating an alias to an already existing [UniqueIdentifier] (UID)
```
use std::sync::Arc;
use gmt_dos_actors::{
    io::{Data, Write, UniqueIdentifier},
    Size, UID
};
use gmt_dos_actors as dos_actors;

// Original UID
#[derive(UID)]
#[uid(data = "u8")]
pub enum A {}
pub struct Client {}
impl Write<A> for Client {
    fn write(&mut self) -> Option<Arc<Data<A>>> {
        Some(Arc::new(Data::new(10u8)))
    }
}
impl Size<A> for Client {
    fn len(&self) -> usize {
        123
    }
}

// A alias with `Write` and `Size` trait implementation for `Client`
#[derive(UID)]
#[alias(name = "A", client = "Client", traits = "Write,Size")]
pub enum B {}

let _: <A as UniqueIdentifier>::Data = 1u8;
let _: <B as UniqueIdentifier>::Data = 2u8;

let mut client = Client {};
 println!(
    "Client Write<B>: {:?}",
    <Client as Write<B>>::write(&mut client)
);
println!(
    "Client Size<B>: {:?}",
    <Client as Size<B>>::len(&mut client)
);
```

[Actor]: crate::actor
*/

use crate::{
    io::{Data, Read, UniqueIdentifier, Write},
    Update,
};
use linya::{Bar, Progress};
use std::{
    mem::take,
    sync::{Arc, Mutex},
};

mod signals;
#[doc(inline)]
pub use signals::{OneSignal, Signal, Signals};
mod timer;
#[doc(inline)]
pub use timer::{Tick, Timer, Void};
mod logging;
#[doc(inline)]
pub use logging::Logging;
mod sampler;
#[doc(inline)]
pub use sampler::Sampler;
mod integrator;
#[doc(inline)]
pub use integrator::Integrator;
mod smooth;
#[doc(inline)]
pub use smooth::{Smooth, Weight};
mod average;
#[doc(inline)]
pub use average::Average;

#[derive(Debug)]
pub(crate) struct ProgressBar {
    progress: Arc<Mutex<Progress>>,
    bar: Bar,
}

pub trait TimerMarker {}
impl<T: TimerMarker> Read<Tick> for T {
    fn read(&mut self, _: Arc<Data<Tick>>) {}
}

/// Concatenates data into a [Vec]
pub struct Concat<T>(Vec<T>);
impl<T: Default> Default for Concat<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
impl<T> Update for Concat<T> {}
impl<T: Clone + Default, U: UniqueIdentifier<Data = T>> Read<U> for Concat<T> {
    fn read(&mut self, data: Arc<Data<U>>) {
        self.0.push((*data).clone());
    }
}
impl<T: Clone, U: UniqueIdentifier<Data = Vec<T>>> Write<U> for Concat<T> {
    fn write(&mut self) -> Option<Arc<Data<U>>> {
        Some(Arc::new(Data::new(take(&mut self.0))))
    }
}

/// Discrete data sets
pub struct Source<T> {
    n: usize,
    data: Vec<T>,
}
impl<T> Source<T> {
    pub fn new(data: Vec<T>, n: usize) -> Self {
        Source { n, data }
    }
}
impl<T> Update for Source<T> {}

impl<T, V> Write<V> for Source<T>
where
    V: UniqueIdentifier<Data = Vec<T>>,
{
    fn write(&mut self) -> Option<Arc<Data<V>>> {
        if self.data.is_empty() {
            None
        } else {
            let y: Vec<T> = self.data.drain(..self.n).collect();
            Some(Arc::new(Data::new(y)))
        }
    }
}

#[cfg(feature = "nalgebra")]
mod gain;
#[cfg(feature = "nalgebra")]
pub use gain::Gain;
