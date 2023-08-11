use std::any::type_name;

use gmt_dos_clients::interface::{Data, UniqueIdentifier};
use serde::{Deserialize, Serialize};

/// Scope client/server payload
///
/// The data type that is sent from a server application to the scope client
#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Payload {
    Signal {
        tag: String,
        tau: f64,
        value: f64,
    },
    Image {
        tag: String,
        size: [usize; 2],
        pixels: Vec<f64>,
        minmax: Option<(f64, f64)>,
    },
}

impl Payload {
    /// Creates a new [Payload] for a signal
    pub fn signal<T, U>(data: Data<U>, tau: f64, idx: Option<usize>) -> Option<Self>
    where
        T: Copy,
        U: UniqueIdentifier<DataType = Vec<T>>,
        f64: From<T>,
    {
        data.get(idx.unwrap_or_default()).map(|&v| Self::Signal {
            tag: type_name::<U>().rsplit("::").next().unwrap().to_owned(),
            tau,
            value: v.into(),
        })
    }
    /// Creates a new [Payload] for an image
    pub fn image<T, U>(data: Data<U>, size: [usize; 2], minmax: Option<(f64, f64)>) -> Option<Self>
    where
        T: Copy,
        U: UniqueIdentifier<DataType = Vec<T>>,
        f64: From<T>,
    {
        Some(Self::Image {
            tag: type_name::<U>().rsplit("::").next().unwrap().to_owned(),
            size,
            pixels: Vec::from(data).into_iter().map(|x| f64::from(x)).collect(),
            minmax,
        })
    }
}

impl Payload {
    pub fn max(&self) -> f64 {
        match self {
            Payload::Image { pixels, .. } => {
                *(pixels.iter().max_by(|&a, &b| a.total_cmp(b)).unwrap())
            }
            _ => unimplemented!("max is implemented only for Payload::Image"),
        }
    }
    pub fn min(&self) -> f64 {
        match self {
            Payload::Image { pixels, .. } => {
                *(pixels.iter().min_by(|&a, &b| a.total_cmp(b)).unwrap())
            }
            _ => unimplemented!("min is implemented only for Payload::Image"),
        }
    }
}
