//! [Actor](crate::Actor)s clients interfaces
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

pub mod signals;
use std::{any::type_name, fmt::Display, sync::Arc};

use crate::{
    io::{Data, Read, Write},
    Update,
};
pub use signals::{Signal, Signals};

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
    pub fn n_entry(self, n_entry: usize) -> Self {
        Self { n_entry, ..self }
    }
    pub fn capacity(self, capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            ..self
        }
    }
}

impl<T> Logging<T> {
    pub fn len(&self) -> usize {
        self.n_sample / self.n_entry
    }
    pub fn n_data(&self) -> usize {
        self.data.len() / self.len()
    }
    pub fn is_empty(&self) -> bool {
        self.n_sample == 0
    }
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

/// Sample-and-hold rate transionner
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
