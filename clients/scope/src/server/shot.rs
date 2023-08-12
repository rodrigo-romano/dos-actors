use std::marker::PhantomData;

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update};
use gmt_dos_clients_transceiver::{Monitor, On, Transceiver, TransceiverError, Transmitter};

use crate::payload::Payload;

#[derive(Debug, thiserror::Error)]
pub enum ShotError {
    #[error("failed to create a transmiter for a scope server")]
    Transmitter(#[from] TransceiverError),
}

pub(crate) struct ScopeData<U: UniqueIdentifier>(PhantomData<U>);
impl<U: UniqueIdentifier> UniqueIdentifier for ScopeData<U> {
    type DataType = Payload;
}

/// [Shot] builder
#[derive(Debug)]
pub struct ShotBuilder<'a, FU>
where
    FU: UniqueIdentifier,
{
    address: String,
    monitor: &'a mut Monitor,
    size: [usize; 2],
    minmax: Option<(f64, f64)>,
    payload: PhantomData<FU>,
}
impl<'a, FU> ShotBuilder<'a, FU>
where
    FU: UniqueIdentifier + 'static,
{
    /// Build the [Shot]
    pub fn build(self) -> Result<Shot<FU>, ShotError> {
        Ok(Shot {
            tx: Transceiver::transmitter(self.address)?.run(self.monitor),
            size: self.size,
            minmax: self.minmax,
        })
    }
    /// Sets the minimum and maximum values of the image colormap
    pub fn minmax(mut self, minmax: (f64, f64)) -> Self {
        self.minmax = Some(minmax);
        self
    }
}

/// [Shot](crate::Shot) server
///
/// Wraps a signal into the scope payload before sending it to a [XScope](crate::XScope)
#[derive(Debug)]
pub struct Shot<FU>
where
    FU: UniqueIdentifier,
{
    tx: Transceiver<ScopeData<FU>, Transmitter, On>,
    size: [usize; 2],
    minmax: Option<(f64, f64)>,
}

impl<FU> Shot<FU>
where
    FU: UniqueIdentifier + 'static,
    <FU as UniqueIdentifier>::DataType: Send + Sync + serde::Serialize,
{
    /// Creates a [ShotBuilder]
    pub fn builder(
        address: impl Into<String>,
        monitor: &mut Monitor,
        size: [usize; 2],
    ) -> ShotBuilder<FU> {
        ShotBuilder {
            address: address.into(),
            monitor,
            size,
            minmax: None,
            payload: PhantomData,
        }
    }
}

impl<FU> Update for Shot<FU> where FU: UniqueIdentifier {}

impl<T, FU> Read<FU> for Shot<FU>
where
    FU: UniqueIdentifier<DataType = Vec<T>>,
    T: Copy,
    f64: From<T>,
{
    fn read(&mut self, data: Data<FU>) {
        let payload = Payload::image(data, self.size, self.minmax)
            .expect("failed to create payload from data");
        <Transceiver<ScopeData<FU>, Transmitter, On> as Read<ScopeData<FU>>>::read(
            &mut self.tx,
            Data::new(payload),
        );
    }
}
