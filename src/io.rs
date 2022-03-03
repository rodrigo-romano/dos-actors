//! [Actor](crate::Actor)s [Input]/[Output]

use crate::{ActorError, Result, Who};
use async_trait::async_trait;
use flume::{Receiver, Sender};
use futures::future::join_all;
use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::Mutex;

/// [Input]/[Output] data
///
/// `T` is the type of transferred data and `U` is the data unique identifier (UID)
pub struct Data<T, U>(pub T, pub PhantomData<U>);
impl<T, U> Deref for Data<T, U> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T, U> DerefMut for Data<T, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T, U> Data<T, U> {
    pub fn new(data: T) -> Self {
        Data(data, PhantomData)
    }
}
impl<T, U> From<&Data<Vec<T>, U>> for Vec<T>
where
    T: Clone,
{
    fn from(data: &Data<Vec<T>, U>) -> Self {
        data.to_vec()
    }
}
impl<T, U> From<Vec<T>> for Data<Vec<T>, U> {
    /// Returns data UID
    fn from(u: Vec<T>) -> Self {
        Data(u, PhantomData)
    }
}
impl<T, U> Who<U> for Data<T, U> {}

impl<T: fmt::Debug, U> fmt::Debug for Data<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&self.who()).field("data", &self.0).finish()
    }
}

impl<T: Default, U> Default for Data<Vec<T>, U> {
    fn default() -> Self {
        Data::new(Default::default())
    }
}

pub(crate) type S<T, U> = Arc<Data<T, U>>;

/// Actor data consumer interface
pub trait Read<T, U> {
    fn read(&mut self, data: Arc<Data<T, U>>);
}
/// [Actor](crate::Actor)s input
pub struct Input<C: Read<T, U>, T, U, const N: usize> {
    rx: Receiver<S<T, U>>,
    client: Arc<Mutex<C>>,
}
impl<C: Read<T, U>, T, U, const N: usize> Input<C, T, U, N> {
    /// Creates a new intput from a [Receiver] and data [Default]
    pub fn new(rx: Receiver<S<T, U>>, client: Arc<Mutex<C>>) -> Self {
        Self { rx, client }
    }
}
impl<C: Read<T, U>, T, U, const N: usize> Who<U> for Input<C, T, U, N> {}

#[async_trait]
pub(crate) trait InputObject: Send + Sync {
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
    /// Receives output data
    async fn recv(&mut self) -> Result<()> {
        (*self.client.lock().await).read(self.rx.recv_async().await?);
        Ok(())
    }

    fn who(&self) -> String {
        Who::who(self)
    }
}
/*
impl<C, T, U, const N: usize> From<&Input<C, Vec<T>, U, N>> for Vec<T>
where
    T: Default + Clone,
    C: Consuming<Vec<T>, U>,
{
    fn from(input: &Input<C, Vec<T>, U, N>) -> Self {
        input.data.as_ref().into()
    }
}
*/
/// Actor data producer interface
pub trait Write<T, U> {
    fn write(&mut self) -> Option<Arc<Data<T, U>>>;
}

/// [Actor](crate::Actor)s output
pub struct Output<C: Write<T, U>, T, U, const N: usize> {
    data: Option<S<T, U>>,
    tx: Vec<Sender<S<T, U>>>,
    client: Arc<Mutex<C>>,
}
impl<C: Write<T, U>, T, U, const N: usize> Output<C, T, U, N> {
    /// Creates a new output from a [Sender] and data [Default]
    pub fn new(tx: Vec<Sender<S<T, U>>>, client: Arc<Mutex<C>>) -> Self {
        Self {
            data: None,
            tx,
            client,
        }
    }
}
impl<C: Write<T, U>, T, U, const N: usize> Who<U> for Output<C, T, U, N> {}

#[async_trait]
pub(crate) trait OutputObject: Send + Sync {
    async fn send(&mut self) -> Result<()>;
    fn who(&self) -> String;
}
#[async_trait]
impl<C, T, U, const N: usize> OutputObject for Output<C, T, U, N>
where
    C: Write<T, U> + Send,
    T: Send + Sync,
    U: Send + Sync,
{
    /// Sends output data
    async fn send(&mut self) -> Result<()> {
        self.data = (*self.client.lock().await).write();
        if let Some(data) = &self.data {
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
            Ok(())
        } else {
            for tx in &self.tx {
                drop(tx);
            }
            Err(ActorError::Disconnected)
        }
    }

    fn who(&self) -> String {
        Who::who(self)
    }
}
