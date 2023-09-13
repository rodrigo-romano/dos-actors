/*!
# Scope server (features = `"server"`)

The scopes defined in the server module send data to the scope clients.

## Example

```ignore
use gmt_dos_clients_scope::server;

#[derive(interface::UID)]
pub enum Signal {}

let server_address = "127.0.0.1:5001";
let sampling_period = 1e-3; // 1ms

let mut monitor = server::Monitor::new();
let server = server::Scope::<Signal>::builder(server_address, &mut monitor)
    .sampling_period(sampling_period)
    .build().unwrap();
```
*/

mod scope;
mod shot;
use std::marker::PhantomData;

pub use gmt_dos_clients_transceiver::Monitor;

use gmt_dos_clients_transceiver::{On, Transceiver, TransceiverError, Transmitter};
use interface::UniqueIdentifier;
pub use shot::{GmtShot, Shot};

use crate::{payload::ScopeData, PlotScope};

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("failed to create a transmiter for a scope server")]
    Transmitter(#[from] TransceiverError),
}

/// Builder for scope server
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
    frame_by_frame: bool,
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
            frame_by_frame: false,
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

/// Server for signal for plotting scope
pub type Scope<FU> = XScope<FU, crate::PlotScope>;

/// Scope server
///
/// Wraps a signal into the scope payload before sending it to the scope [client](crate::client)
#[derive(Debug)]
pub struct XScope<FU, K = PlotScope>
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

impl<FU, K: crate::ScopeKind> interface::Update for XScope<FU, K> where FU: UniqueIdentifier {}
