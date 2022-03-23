use super::{Read, S};
use crate::{Result, Who};
use async_trait::async_trait;
use flume::Receiver;
use std::sync::Arc;
use tokio::sync::Mutex;

/// [Actor](crate::Actor)s input
pub(crate) struct Input<C: Read<T, U>, T, U, const N: usize> {
    rx: Receiver<S<T, U>>,
    client: Arc<Mutex<C>>,
}
impl<C: Read<T, U>, T, U, const N: usize> Input<C, T, U, N> {
    /// Creates a new intput from a [Receiver] and an [Actor] client
    pub fn new(rx: Receiver<S<T, U>>, client: Arc<Mutex<C>>) -> Self {
        Self { rx, client }
    }
}
impl<C: Read<T, U>, T, U, const N: usize> Who<U> for Input<C, T, U, N> {}

#[async_trait]
pub(crate) trait InputObject: Send + Sync {
    /// Receives output data
    async fn recv(&mut self) -> Result<()>;
    fn who(&self) -> String;
}

#[async_trait]
impl<C, T, U, const N: usize> InputObject for Input<C, T, U, N>
where
    C: Read<T, U> + Send,
    T: Send + Sync,
    U: Send + Sync,
{
    async fn recv(&mut self) -> Result<()> {
        log::debug!("{} receiving", Who::who(self));
        log::debug!("{} receiving (locking client)", Who::who(self));
        let mut client = self.client.lock().await;
        log::debug!("{} receiving (client locked)", Who::who(self));
        (*client).read(self.rx.recv_async().await?);
        log::debug!("{} received", Who::who(self));
        Ok(())
    }
    fn who(&self) -> String {
        Who::who(self)
    }
}
