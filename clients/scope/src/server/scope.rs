use std::marker::PhantomData;

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update};
use gmt_dos_clients_transceiver::{Monitor, On, Transceiver, Transmitter};

use crate::{
    payload::{Payload, ScopeData},
    ScopeKind,
};

use super::Scope;

impl<'a, FU> super::Builder<'a, FU, crate::PlotScope>
where
    FU: UniqueIdentifier + 'static,
{
    /// Selects the signal channel #
    pub fn channel(mut self, idx: usize) -> Self {
        self.idx = Some(idx);
        self
    }
    /// Build the [Scope]
    pub fn build(self) -> Result<Scope<FU>, super::ServerError> {
        Ok(Scope {
            tx: Transceiver::transmitter(self.address)?.run(self.monitor.unwrap()),
            tau: self.tau.unwrap_or(1f64),
            idx: self.idx.unwrap_or_default(),
            scale: self.scale,
            size: [0; 2],
            minmax: None,
            kind: PhantomData,
        })
    }
}

impl<FU> Scope<FU>
where
    FU: UniqueIdentifier + 'static,
    <FU as UniqueIdentifier>::DataType: Send + Sync + serde::Serialize,
{
    /// Creates a [ScopeBuilder]
    pub fn builder(
        address: impl Into<String>,
        monitor: &mut Monitor,
    ) -> super::Builder<FU, crate::PlotScope> {
        super::Builder {
            address: address.into(),
            monitor: Some(monitor),
            ..Default::default()
        }
    }
}

impl<FU, K: ScopeKind> Update for Scope<FU, K> where FU: UniqueIdentifier {}

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
