use super::{Read, S};
use crate::{ActorError, Result, UniqueIdentifier, Who};
use async_trait::async_trait;
use flume::Receiver;
use std::{fmt::Display, sync::Arc};
use tokio::sync::Mutex;

/// [Actor](crate::Actor)s input
pub(crate) struct Input<C, T, U, const N: usize>
where
    U: UniqueIdentifier<DataType = T>,
    C: Read<U>,
{
    rx: Receiver<S<U>>,
    client: Arc<Mutex<C>>,
    hash: u64,
}
impl<C, T, U, const N: usize> Input<C, T, U, N>
where
    U: UniqueIdentifier<DataType = T>,
    C: Read<U>,
{
    /// Creates a new intput from a [Receiver], an [Actor] client and an identifier [hash]
    pub fn new(rx: Receiver<S<U>>, client: Arc<Mutex<C>>, hash: u64) -> Self {
        Self { rx, client, hash }
    }
}
impl<C, T, U, const N: usize> Who<U> for Input<C, T, U, N>
where
    C: Read<U>,
    U: UniqueIdentifier<DataType = T>,
{
}
impl<C, T, U, const N: usize> Display for Input<C, T, U, N>
where
    C: Read<U>,
    U: UniqueIdentifier<DataType = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:>19}: {}", self.hash, self.who())
    }
}

#[async_trait]
pub(crate) trait InputObject: Display + Send + Sync {
    /// Receives output data
    async fn recv(&mut self) -> Result<()>;
    /// Returns the input UID
    fn who(&self) -> String;
    /// Gets the input hash
    fn get_hash(&self) -> u64;
    fn capacity(&self) -> Option<usize>;
}

#[async_trait]
impl<C, T, U, const N: usize> InputObject for Input<C, T, U, N>
where
    C: Read<U> + Send,
    T: Send + Sync,
    U: Send + Sync + UniqueIdentifier<DataType = T>,
{
    async fn recv(&mut self) -> Result<()> {
        log::debug!("{} receiving", Who::highlight(self));
        // log::debug!("{} receiving (locking client)", Who::who(self));
        let mut client = self.client.lock().await;
        // log::debug!("{} receiving (client locked)", Who::who(self));
        (*client).read(
            self.rx
                .recv_async()
                .await
                .map_err(|e| ActorError::DropRecv {
                    msg: Who::lite(self),
                    source: e,
                })?,
        );
        log::debug!("{} received", Who::highlight(self));
        Ok(())
    }
    fn who(&self) -> String {
        Who::who(self)
    }
    fn get_hash(&self) -> u64 {
        self.hash
    }
    fn capacity(&self) -> Option<usize> {
        self.rx.capacity()
    }
}
