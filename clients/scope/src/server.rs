mod scope;
mod shot;
use std::marker::PhantomData;

use gmt_dos_clients::interface::UniqueIdentifier;
use gmt_dos_clients_transceiver::{Monitor, On, Transceiver, TransceiverError, Transmitter};
pub use shot::{GmtShot, Shot};

use crate::{payload::ScopeData, PlotScope};

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("failed to create a transmiter for a scope server")]
    Transmitter(#[from] TransceiverError),
}

/// [Scope] builder
#[derive(Debug)]
pub struct Builder<'a, FU, K>
where
    FU: UniqueIdentifier,
    K: crate::ScopeKind,
{
    address: String,
    monitor: Option<&'a mut Monitor>,
    tau: Option<f64>,
    idx: Option<usize>,
    scale: Option<f64>,
    size: Option<[usize; 2]>,
    minmax: Option<(f64, f64)>,
    payload: PhantomData<FU>,
    kind: PhantomData<K>,
}

impl<'a, FU, K> Default for Builder<'a, FU, K>
where
    FU: UniqueIdentifier,
    K: crate::ScopeKind,
{
    fn default() -> Self {
        Self {
            address: Default::default(),
            monitor: Default::default(),
            tau: Default::default(),
            idx: Default::default(),
            scale: Default::default(),
            size: Default::default(),
            minmax: Default::default(),
            payload: PhantomData,
            kind: PhantomData,
        }
    }
}

impl<'a, FU, K> Builder<'a, FU, K>
where
    FU: UniqueIdentifier + 'static,
    K: crate::ScopeKind,
{
    /// Sets the signal sampling period
    pub fn sampling_period(mut self, tau: f64) -> Self {
        self.tau = Some(tau);
        self
    }
    /// Sets the factor to scale up the data
    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = Some(scale);
        self
    }
}

/// [Scope](crate::Scope) server
///
/// Wraps a signal into the scope payload before sending it to a [XScope](crate::XScope)
#[derive(Debug)]
pub struct Scope<FU, K = PlotScope>
where
    FU: UniqueIdentifier,
{
    tx: Transceiver<ScopeData<FU>, Transmitter, On>,
    tau: f64,
    idx: usize,
    size: [usize; 2],
    minmax: Option<(f64, f64)>,
    scale: Option<f64>,
    kind: PhantomData<K>,
}
