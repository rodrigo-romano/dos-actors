use super::S;
use crate::interface::{Assoc, Write};
use crate::{ActorError, Result, UniqueIdentifier, Who};
use async_trait::async_trait;
use flume::Sender;
use futures::future::join_all;
use std::any::{type_name, Any};
use std::fmt::Debug;
use std::{fmt::Display, sync::Arc};
use tokio::sync::Mutex;

pub(crate) struct OutputBuilder<C, T, U, const N: usize>
where
    U: UniqueIdentifier<DataType = T>,
    C: Write<U>,
{
    tx: Vec<Sender<S<U>>>,
    client: Arc<Mutex<C>>,
    bootstrap: bool,
}
impl<C, T, U, const N: usize> OutputBuilder<C, T, U, N>
where
    U: UniqueIdentifier<DataType = T>,
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
    U: UniqueIdentifier<DataType = T>,
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
    U: UniqueIdentifier<DataType = T>,
    C: Write<U>,
{
    /// Creates a new output from a [Sender] and data [Default]
    pub fn builder(client: Arc<Mutex<C>>) -> OutputBuilder<C, T, U, N> {
        OutputBuilder::new(client)
    }
    pub fn tx_push(&mut self, mut tx: Vec<Sender<S<U>>>) -> &mut Self {
        self.tx.append(&mut tx);
        self
    }
}
impl<C, T, U, const N: usize> Who<U> for Output<C, T, U, N>
where
    C: Write<U>,
    U: UniqueIdentifier<DataType = T>,
{
}
impl<C, T, U, const N: usize> Display for Output<C, T, U, N>
where
    C: Write<U> + Send + 'static,
    T: Send + Sync + 'static,
    U: UniqueIdentifier<DataType = T> + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:>19}: {} x{} {}",
            self.hash,
            Who::who(self),
            self.len(),
            self.bootstrap.then_some("(bootstrap)").unwrap_or_default()
        )
    }
}
impl<C: Write<U> + Debug, T: Debug, U: UniqueIdentifier<DataType = T>, const N: usize> Debug
    for Output<C, T, U, N>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Output")
            .field("data", &self.data)
            .field("tx", &self.tx)
            .field("client", &self.client)
            .field("bootstrap", &self.bootstrap)
            .field("hash", &self.hash)
            .finish()
    }
}

#[async_trait]
pub(crate) trait OutputObject: Any + Display + Send + Sync {
    async fn send(&mut self) -> Result<()>;
    fn bootstrap(&self) -> bool;
    fn len(&self) -> usize;
    fn who(&self) -> String;
    fn highlight(&self) -> String;
    fn set_hash(&mut self, hash: u64);
    fn get_hash(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl Debug for Box<dyn OutputObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&std::ops::Deref::deref(&self), f)
    }
}

#[async_trait]
impl<C, T, U, const N: usize> OutputObject for Output<C, T, U, N>
where
    C: Write<U> + Send + 'static,
    T: Send + Sync + 'static,
    U: Send + Sync + UniqueIdentifier<DataType = T> + 'static,
    Assoc<U>: Send + Sync,
{
    /// Sends output data
    async fn send(&mut self) -> Result<()> {
        self.data = (*self.client.lock().await).write();
        if let Some(data) = &self.data {
            log::debug!("{} sending", Who::highlight(self));
            let futures: Vec<_> = self
                .tx
                .iter()
                .map(|tx| tx.send_async(data.clone()))
                .collect();
            join_all(futures)
                .await
                .into_iter()
                .collect::<std::result::Result<Vec<()>, flume::SendError<_>>>()
                .map_err(|_| ActorError::DropSend {
                    msg: Who::lite(self),
                    source: flume::SendError(()),
                })?;
            log::debug!("{} sent ({})", Who::highlight(self), type_name::<C>());
            Ok(())
        } else {
            for tx in std::mem::replace(&mut self.tx, vec![]) {
                drop(tx);
            }
            Err(ActorError::Disconnected(Who::lite(self)))
        }
    }
    /// Bootstraps output
    fn bootstrap(&self) -> bool {
        self.bootstrap
    }
    fn who(&self) -> String {
        Who::who(self)
    }
    fn highlight(&self) -> String {
        Who::highlight(self)
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
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
