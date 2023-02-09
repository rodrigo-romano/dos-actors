use crate::{
    io::{self, Size, Update},
    Actor, ActorError, Result, UniqueIdentifier, Who,
};
use async_trait::async_trait;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use super::{Entry, IntoInputs, IntoLogs, IntoLogsN};

// Unique hash for a pair of input/output
fn hashio<CO, const NO: usize, const NI: usize>(output_actor: &mut Actor<CO, NI, NO>) -> u64
where
    CO: Update + Send,
{
    let mut hasher = DefaultHasher::new();
    output_actor.who().hash(&mut hasher);
    let output = output_actor
        .outputs
        .as_mut()
        .and_then(|o| o.last_mut())
        .unwrap();
    output
        .who()
        .split("::")
        .last()
        .unwrap()
        .to_owned()
        .hash(&mut hasher);
    let hash = hasher.finish();
    output.set_hash(hash);
    hash
}
impl<'a, T, U, CO, const NO: usize, const NI: usize> IntoInputs<'a, T, U, CO, NO, NI>
    for (
        &'a mut Actor<CO, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<U>>>>,
    )
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
    CO: 'static + Update + Send + io::Write<U>,
{
    fn into_input<CI, const N: usize>(mut self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        CI: 'static + Update + Send + io::Read<U>,
    {
        if let Some(recv) = self.1.pop() {
            actor.add_input(recv, hashio(self.0))
        }
        self
    }
    fn ok(self) -> Result<&'a mut Actor<CO, NI, NO>> {
        if self.1.is_empty() {
            Ok(self.0)
        } else {
            Err(ActorError::OrphanOutput(
                self.0.outputs.as_ref().unwrap().last().unwrap().who(),
                self.0.who(),
            ))
        }
    }
}

#[async_trait]
impl<T, U, CI, CO, const N: usize, const NO: usize, const NI: usize> IntoLogsN<CI, N, NO>
    for (
        &mut Actor<CO, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<U>>>>,
    )
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
    CI: 'static + Update + Send + io::Read<U> + Entry<U>,
    CO: 'static + Update + Send + io::Write<U>,
{
    /// Creates a new logging entry for the output
    async fn logn(mut self, actor: &mut Actor<CI, NO, N>, size: usize) -> Self {
        if let Some(recv) = self.1.pop() {
            (*actor.client.lock().await).entry(size);
            actor.add_input(recv, hashio(self.0))
        }
        self
    }
}

#[async_trait]
impl<T, U, CI, CO, const N: usize, const NO: usize, const NI: usize> IntoLogs<CI, N, NO>
    for (
        &mut Actor<CO, NI, NO>,
        Vec<flume::Receiver<Arc<io::Data<U>>>>,
    )
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
    CI: 'static + Update + Send + io::Read<U> + Entry<U>,
    CO: 'static + Update + Send + io::Write<U> + Size<U>,
{
    /// Creates a new logging entry for the output
    async fn log(mut self, actor: &mut Actor<CI, NO, N>) -> Self {
        if let Some(recv) = self.1.pop() {
            (*actor.client.lock().await)
                .entry(<CO as Size<U>>::len(&mut *self.0.client.lock().await));
            actor.add_input(recv, hashio(self.0))
        }
        self
    }
}
