use std::marker::PhantomData;

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update};
use gmt_dos_clients_transceiver::{Monitor, On, Transceiver, TransceiverError, Transmitter};

use crate::payload::Payload;

#[derive(Debug, thiserror::Error)]
pub enum ScopeServerError {
    #[error("failed to create a transmiter for a scope server")]
    Transmitter(#[from] TransceiverError),
}

pub(crate) struct ScopeData<U: UniqueIdentifier>(PhantomData<U>);
impl<U: UniqueIdentifier> UniqueIdentifier for ScopeData<U> {
    type DataType = Payload;
}

/// [ScopeServer] builder
#[derive(Debug)]
pub struct ScopeServerBuilder<'a, FU>
where
    FU: UniqueIdentifier,
{
    address: String,
    monitor: &'a mut Monitor,
    tau: Option<f64>,
    idx: Option<usize>,
    payload: PhantomData<FU>,
}
impl<'a, FU> ScopeServerBuilder<'a, FU>
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
    /// Build the [ScopeServer]
    pub fn build(self) -> Result<ScopeServer<FU>, ScopeServerError> {
        Ok(ScopeServer {
            tx: Transceiver::transmitter(self.address)?.run(self.monitor),
            tau: self.tau.unwrap_or(1f64),
            idx: self.idx.unwrap_or_default(),
        })
    }
}

/// Scope server
///
/// Wraps a signal into the scope payload before sending it to a [XScope](crate::XScope)
#[derive(Debug)]
pub struct ScopeServer<FU>
where
    FU: UniqueIdentifier,
{
    tx: Transceiver<ScopeData<FU>, Transmitter, On>,
    tau: f64,
    idx: usize,
}

impl<FU> ScopeServer<FU>
where
    FU: UniqueIdentifier + 'static,
    <FU as UniqueIdentifier>::DataType: Send + Sync + serde::Serialize,
{
    /// Creates a [ScopeServerBuilder]
    pub fn builder(address: impl Into<String>, monitor: &mut Monitor) -> ScopeServerBuilder<FU> {
        ScopeServerBuilder {
            address: address.into(),
            monitor,
            tau: None,
            idx: None,
            payload: PhantomData,
        }
    }
}

impl<FU> Update for ScopeServer<FU> where FU: UniqueIdentifier {}

impl<T, FU> Read<FU> for ScopeServer<FU>
where
    FU: UniqueIdentifier<DataType = Vec<T>>,
    T: Copy,
    f64: From<T>,
{
    fn read(&mut self, data: Data<FU>) {
        let payload = Payload::signal(data, self.tau, Some(self.idx))
            .expect("failed to create payload from data");
        <Transceiver<ScopeData<FU>, Transmitter, On> as Read<ScopeData<FU>>>::read(
            &mut self.tx,
            Data::new(payload),
        );
    }
}
