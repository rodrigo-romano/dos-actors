use std::marker::PhantomData;

use gmt_dos_clients_transceiver::{Monitor, On, Transceiver, Transmitter};
use interface::{Data, Read, UniqueIdentifier};

use crate::payload::{Payload, ScopeData};

use super::Scope;

impl<'a, FU> super::Builder<'a, FU, crate::PlotScope>
where
    FU: UniqueIdentifier + 'static,
{
    /// Build the [Scope]
    pub fn build(self) -> Result<Scope<FU>, super::ServerError> {
        Ok(Scope {
            tx: Transceiver::transmitter(self.address)?.run(self.monitor.unwrap()),
            tau: self.tau.unwrap_or(1f64),
            idx: self.idx,
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
    /// Creates a [Builder](super::Builder)
    pub fn builder(monitor: &mut Monitor) -> super::Builder<FU, crate::PlotScope> {
        super::Builder {
            monitor: Some(monitor),
            ..Default::default()
        }
    }
}

impl<T, FU> Read<FU> for Scope<FU>
where
    FU: UniqueIdentifier<DataType = Vec<T>>,
    T: Copy,
    f64: From<T>,
{
    fn read(&mut self, data: Data<FU>) {
        let payload = Payload::signal(data, self.tau, self.idx, self.scale)
            .expect("failed to create payload from data");
        <Transceiver<ScopeData<FU>, Transmitter, On> as Read<ScopeData<FU>>>::read(
            &mut self.tx,
            Data::new(payload),
        );
    }
}
