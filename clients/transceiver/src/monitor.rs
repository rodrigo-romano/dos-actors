use std::{
    future::IntoFuture,
    ops::{Deref, DerefMut},
};

use tokio::task::{JoinError, JoinHandle};

use crate::TransceiverError;

/// [Transceiver](crate::Transceiver) monitor
///
/// Collect [Transceiver](crate::Transceiver) transmitter or receiver thread handles
///
#[derive(Default, Debug)]
pub struct Monitor(Vec<JoinHandle<crate::Result<()>>>);
impl Monitor {
    /// Creates a new empty [Transceiver](crate::Transceiver) monitor
    pub fn new() -> Self {
        Default::default()
    }
    /// Joins all [Transceiver](crate::Transceiver) threads
    ///
    /// Instead you can `await` on [Monitor]s
    pub async fn join(self) -> crate::Result<()> {
        for h in self.0 {
            let _ = h.await?;
        }
        Ok(())
    }
}
impl Deref for Monitor {
    type Target = Vec<JoinHandle<crate::Result<()>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Monitor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoFuture for Monitor {
    type Output = Vec<Result<Result<(), TransceiverError>, JoinError>>;

    type IntoFuture = futures::future::JoinAll<JoinHandle<crate::Result<()>>>;

    fn into_future(self) -> Self::IntoFuture {
        futures::future::join_all(self.0)
    }
}
