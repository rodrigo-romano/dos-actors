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

#[cfg(feature = "m1-ctrl")]
pub mod m1;

#[cfg(feature = "apache-arrow")]
pub mod arrow_client;

pub mod signals;
use std::{any::type_name, sync::Arc};

use crate::{
    io::{Data, Read},
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

/// Client method specifications
pub trait Client {
    //: std::fmt::Debug {
    type I;
    type O;
    /// Processes the [Actor](crate::Actor) [inputs](crate::Actor::inputs) for the client
    fn read(&mut self, _data: Vec<&Self::I>) -> &mut Self {
        self
    }
    /// Generates the [outputs](crate::Actor::outputs) from the client
    fn write(&mut self) -> Option<Vec<Self::O>> {
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
impl<T> Update for Logging<T> {}
impl<T: Clone, U> Read<Vec<T>, U> for Logging<T> {
    fn read(&mut self, data: Arc<Data<Vec<T>, U>>) {
        log::debug!("receive {} input: {:}", type_name::<U>(), data.len(),);
        self.0.extend((**data).clone());
    }
}

/// Sample-and-hold rate transionner
#[derive(Debug, Default)]
pub struct Sampler<T>(Vec<T>);
impl<T> Client for Sampler<T>
where
    T: std::fmt::Debug + Clone,
{
    type I = T;
    type O = T;
    fn read(&mut self, data: Vec<&Self::I>) -> &mut Self {
        self.0 = data.into_iter().cloned().collect();
        self
    }
    fn write(&mut self) -> Option<Vec<Self::O>> {
        Some(self.0.drain(..).collect())
    }
}
