use std::marker::PhantomData;

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update};
use gmt_dos_clients_transceiver::{Monitor, On, Transceiver, TransceiverError, Transmitter};

use crate::payload::{Payload, ScopeData};

#[derive(Debug, thiserror::Error)]
pub enum ScopeError {
    #[error("failed to create a transmiter for a scope server")]
    Transmitter(#[from] TransceiverError),
}

/// [Scope] builder
#[derive(Debug)]
pub struct ScopeBuilder<'a, FU>
where
    FU: UniqueIdentifier,
{
    address: String,
    monitor: &'a mut Monitor,
    tau: Option<f64>,
    idx: Option<usize>,
    scale: Option<f64>,
    payload: PhantomData<FU>,
}
impl<'a, FU> ScopeBuilder<'a, FU>
where
    FU: UniqueIdentifier + 'static,
{
    /// Sets the signal sampling period
    pub fn sampling_period(mut self, tau: f64) -> Self {
        self.tau = Some(tau);
        self
    }
    /// Selects the signal channel #
    pub fn channel(mut self, idx: usize) -> Self {
        self.idx = Some(idx);
        self
    }
    /// Sets the factor to scale up the data
    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = Some(scale);
        self
    }
    /// Build the [Scope]
    pub fn build(self) -> Result<Scope<FU>, ScopeError> {
        Ok(Scope {
            tx: Transceiver::transmitter(self.address)?.run(self.monitor),
            tau: self.tau.unwrap_or(1f64),
            idx: self.idx.unwrap_or_default(),
            scale: self.scale,
        })
    }
}

/// [Scope](crate::Scope) server
///
/// Wraps a signal into the scope payload before sending it to a [XScope](crate::XScope)
#[derive(Debug)]
pub struct Scope<FU>
where
    FU: UniqueIdentifier,
{
    tx: Transceiver<ScopeData<FU>, Transmitter, On>,
    tau: f64,
    idx: usize,
    scale: Option<f64>,
}

impl<FU> Scope<FU>
where
    FU: UniqueIdentifier + 'static,
    <FU as UniqueIdentifier>::DataType: Send + Sync + serde::Serialize,
{
    /// Creates a [ScopeBuilder]
    pub fn builder(address: impl Into<String>, monitor: &mut Monitor) -> ScopeBuilder<FU> {
        ScopeBuilder {
            address: address.into(),
            monitor,
            tau: None,
            idx: None,
            scale: None,
            payload: PhantomData,
        }
    }
}

impl<FU> Update for Scope<FU> where FU: UniqueIdentifier {}

impl<T, FU> Read<FU> for Scope<FU>
where
    FU: UniqueIdentifier<DataType = Vec<T>>,
    T: Copy,
    f64: From<T>,
{
    fn read(&mut self, data: Data<FU>) {
        let payload = Payload::signal(data, self.tau, Some(self.idx), self.scale)
            .expect("failed to create payload from data");
        <Transceiver<ScopeData<FU>, Transmitter, On> as Read<ScopeData<FU>>>::read(
            &mut self.tx,
            Data::new(payload),
        );
    }
}
