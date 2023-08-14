use gmt_dos_clients::interface::UniqueIdentifier;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// [ScopeData] is the unique identifier type
/// for the data that holds the scope [Payload]
pub(crate) struct ScopeData<U: UniqueIdentifier>(PhantomData<U>);
impl<U: UniqueIdentifier> UniqueIdentifier for ScopeData<U> {
    type DataType = Payload;
}

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
        mask: Option<Vec<bool>>,
        minmax: Option<(f64, f64)>,
    },
}

#[cfg(feature = "server")]
impl Payload {
    /// Creates a new [Payload] for a signal
    pub fn signal<T, U>(
        data: gmt_dos_clients::interface::Data<U>,
        tau: f64,
        idx: Option<usize>,
        scale: Option<f64>,
    ) -> Option<Self>
    where
        T: Copy,
        U: UniqueIdentifier<DataType = Vec<T>>,
        f64: From<T>,
    {
        data.get(idx.unwrap_or_default()).map(|&v| Self::Signal {
            tag: std::any::type_name::<U>()
                .rsplit("::")
                .next()
                .unwrap()
                .to_owned(),
            tau,
            value: scale.map_or_else(|| v.into(), |s| f64::from(v) * s),
        })
    }
    /// Creates a new [Payload] for an image
    pub fn image<T, U>(
        data: gmt_dos_clients::interface::Data<U>,
        size: [usize; 2],
        minmax: Option<(f64, f64)>,
        scale: Option<f64>,
    ) -> Option<Self>
    where
        T: Copy,
        U: UniqueIdentifier<DataType = Vec<T>>,
        f64: From<T>,
    {
        Some(Self::Image {
            tag: std::any::type_name::<U>()
                .rsplit("::")
                .next()
                .unwrap()
                .to_owned(),
            size,
            pixels: Vec::from(data)
                .into_iter()
                .map(|x| scale.map_or_else(|| x.into(), |s| f64::from(x) * s))
                .collect(),
            mask: None,
            minmax,
        })
    }
    /// Creates a new [Payload] for the GMT wavefront
    pub fn gmt<T, U>(
        data: gmt_dos_clients::interface::Data<U>,
        size: [usize; 2],
        minmax: Option<(f64, f64)>,
        scale: Option<f64>,
    ) -> Option<Self>
    where
        T: Copy,
        U: UniqueIdentifier<DataType = (Vec<T>, Vec<bool>)>,
        f64: From<T>,
    {
        let (pixels, mask) = std::ops::Deref::deref(&data).clone();
        Some(Self::Image {
            tag: std::any::type_name::<U>()
                .rsplit("::")
                .next()
                .unwrap()
                .to_owned(),
            size,
            pixels: Vec::from(pixels)
                .into_iter()
                .map(|x| scale.map_or_else(|| x.into(), |s| f64::from(x) * s))
                .collect(),
            mask: Some(mask),
            minmax,
        })
    }
}

#[cfg(not(feature = "server"))]
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
