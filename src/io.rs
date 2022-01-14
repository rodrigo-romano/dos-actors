//! [Actor](crate::Actor)s [Input]/[Output]

use crate::Result;
use flume::{Receiver, Sender};
use futures::future::join_all;
use std::{ops::Deref, sync::Arc};

/// [Input]/[Output] data
///
/// `N` is the data transfer rate
#[derive(Debug)]
pub struct Data<T, const N: usize>(pub T);
impl<T, const N: usize> Deref for Data<T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Clone, const N: usize> From<&Data<Vec<T>, N>> for Vec<T> {
    fn from(data: &Data<Vec<T>, N>) -> Self {
        data.to_vec()
    }
}
impl<T, const N: usize> From<Vec<T>> for Data<Vec<T>, N> {
    fn from(u: Vec<T>) -> Self {
        Data(u)
    }
}

pub type S<T, const N: usize> = Arc<Data<T, N>>;

/// [Actor](crate::Actor)s input
#[derive(Debug)]
pub struct Input<T: Default, const N: usize> {
    pub data: S<T, N>,
    pub rx: Receiver<S<T, N>>,
}
impl<T: Default, const N: usize> Input<T, N> {
    pub fn new(data: T, rx: Receiver<S<T, N>>) -> Self {
        Self {
            data: Arc::new(Data(data)),
            rx,
        }
    }
    pub async fn recv(&mut self) -> Result<&mut Self> {
        self.data = self.rx.recv_async().await?;
        Ok(self)
    }
}
impl<T: Clone, const N: usize> From<&Input<Vec<T>, N>> for Vec<T> {
    fn from(input: &Input<Vec<T>, N>) -> Self {
        input.data.as_ref().into()
    }
}
/// [Actor](crate::Actor)s output
#[derive(Debug)]
pub struct Output<T: Default, const N: usize> {
    pub data: S<T, N>,
    pub tx: Vec<Sender<S<T, N>>>,
}
impl<T: Default, const N: usize> Output<T, N> {
    pub fn new(data: T, tx: Vec<Sender<S<T, N>>>) -> Self {
        Self {
            data: Arc::new(Data(data)),
            tx,
        }
    }
    pub async fn send(&self) -> Result<&Self> {
        let futures: Vec<_> = self
            .tx
            .iter()
            .map(|tx| tx.send_async(self.data.clone()))
            .collect();
        join_all(futures)
            .await
            .into_iter()
            .collect::<std::result::Result<Vec<()>, flume::SendError<_>>>()
            .map_err(|_| flume::SendError(()))?;
        Ok(self)
    }
}
