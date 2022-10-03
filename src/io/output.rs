use super::{Assoc, Write, S};
use crate::{ActorError, Result, UniqueIdentifier, Who};
use async_trait::async_trait;
use flume::Sender;
use futures::future::join_all;
use std::{fmt::Display, sync::Arc};
use tokio::sync::Mutex;

pub(crate) struct OutputBuilder<C, T, U, const N: usize>
where
    U: UniqueIdentifier<Data = T>,
    C: Write<U>,
{
    tx: Vec<Sender<S<U>>>,
    client: Arc<Mutex<C>>,
    bootstrap: bool,
}
impl<C, T, U, const N: usize> OutputBuilder<C, T, U, N>
where
    U: UniqueIdentifier<Data = T>,
    C: Write<U>,
{
    pub fn new(client: Arc<Mutex<C>>) -> Self {
        Self {
            tx: Vec::new(),
            client,
            bootstrap: false,
        }
    }
    pub fn senders(self, tx: Vec<Sender<S<U>>>) -> Self {
        Self { tx, ..self }
    }
    pub fn bootstrap(self, bootstrap: bool) -> Self {
        Self { bootstrap, ..self }
    }
    pub fn build(self) -> Output<C, T, U, N> {
        Output {
            data: None,
            tx: self.tx,
            client: self.client,
            bootstrap: self.bootstrap,
            hash: 0,
        }
    }
}

/// [Actor](crate::Actor)s output
pub(crate) struct Output<C, T, U, const N: usize>
where
    U: UniqueIdentifier<Data = T>,
    C: Write<U>,
{
    data: Option<S<U>>,
    tx: Vec<Sender<S<U>>>,
    client: Arc<Mutex<C>>,
    bootstrap: bool,
    hash: u64,
}
impl<C, T, U, const N: usize> Output<C, T, U, N>
where
    U: UniqueIdentifier<Data = T>,
    C: Write<U>,
{
    /// Creates a new output from a [Sender] and data [Default]
    pub fn builder(client: Arc<Mutex<C>>) -> OutputBuilder<C, T, U, N> {
        OutputBuilder::new(client)
    }
}
impl<C, T, U, const N: usize> Who<U> for Output<C, T, U, N>
where
    C: Write<U>,
    U: UniqueIdentifier<Data = T>,
{
}
impl<C, T, U, const N: usize> Display for Output<C, T, U, N>
where
    C: Write<U> + Send,
    T: Send + Sync,
    U: UniqueIdentifier<Data = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.bootstrap {
            write!(
                f,
                "{}: {} x{} (bootstrap)",
                self.hash,
                Who::who(self),
                self.len()
            )
        } else {
            write!(f, "{}: {} x{}", self.hash, Who::who(self), self.len())
        }
    }
}

#[async_trait]
pub(crate) trait OutputObject: Display + Send + Sync {
    async fn send(&mut self) -> Result<()>;
    fn bootstrap(&self) -> bool;
    fn len(&self) -> usize;
    fn who(&self) -> String;
    fn set_hash(&mut self, hash: u64);
    fn get_hash(&self) -> u64;
}
#[async_trait]
impl<C, T, U, const N: usize> OutputObject for Output<C, T, U, N>
where
    C: Write<U> + Send,
    T: Send + Sync,
    U: Send + Sync + UniqueIdentifier<Data = T>,
    Assoc<U>: Send + Sync,
{
    /// Sends output data
    async fn send(&mut self) -> Result<()> {
        self.data = (*self.client.lock().await).write();
        if let Some(data) = &self.data {
            log::debug!("{} sending", Who::who(self));
            let futures: Vec<_> = self
                .tx
                .iter()
                .map(|tx| tx.send_async(data.clone()))
                .collect();
            join_all(futures)
                .await
                .into_iter()
                .collect::<std::result::Result<Vec<()>, flume::SendError<_>>>()
                .map_err(|_| flume::SendError(()))?;
            log::debug!("{} sent", Who::who(self));
            Ok(())
        } else {
            for tx in &self.tx {
                drop(tx);
            }
            Err(ActorError::Disconnected(Who::who(self)))
        }
    }
    /// Bootstraps output
    fn bootstrap(&self) -> bool {
        self.bootstrap
    }
    fn who(&self) -> String {
        Who::who(self)
    }

    fn len(&self) -> usize {
        self.tx.len()
    }
    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
    }
    fn get_hash(&self) -> u64 {
        self.hash
    }
}
