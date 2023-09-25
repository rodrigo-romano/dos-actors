use std::any::type_name;

use async_trait::async_trait;
use interface::{Entry, Read, Size, UniqueIdentifier, Update, Write};

use crate::actor::Actor;

use super::{AddActorInput, OutputRx};

/// Assign a new entry to a logging actor
#[async_trait]
pub trait IntoLogsN<CI, const N: usize, const NO: usize>
where
    CI: Update,
{
    async fn logn(mut self, actor: &mut Actor<CI, NO, N>, size: usize) -> Self
    where
        Self: Sized;
}

/// Assign a new entry to a logging actor
#[async_trait]
pub trait IntoLogs<CI, const N: usize, const NO: usize>
where
    CI: Update,
{
    async fn log(self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        Self: Sized;
}

#[async_trait]
impl<T, U, CI, CO, const N: usize, const NO: usize, const NI: usize> IntoLogsN<CI, N, NO>
    for std::result::Result<(), OutputRx<U, CO, NI, NO>>
where
    T: 'static + Send + Sync,
    U: 'static + UniqueIdentifier<DataType = T>,
    CI: 'static + Read<U> + Entry<U>,
    CO: 'static + Write<U>,
{
    /// Creates a new logging entry for the output
    async fn logn(mut self, actor: &mut Actor<CI, NO, N>, size: usize) -> Self {
        match self {
            Ok(()) => panic!(
                r#"Input receivers have been exhausted, may be {} should be multiplexed"#,
                type_name::<U>()
            ),
            Err(OutputRx {
                hash, ref mut rxs, ..
            }) => {
                let Some(recv) = rxs.pop() else {
                    panic!(r#"Input receivers is empty"#)
                };
                (*actor.client.lock().await).entry(size);
                actor.add_input(recv, hash);
                if rxs.is_empty() {
                    Ok(())
                } else {
                    self
                }
            }
        }
    }
}

#[async_trait]
impl<T, U, CI, CO, const N: usize, const NO: usize, const NI: usize> IntoLogs<CI, N, NO>
    for std::result::Result<(), OutputRx<U, CO, NI, NO>>
where
    T: 'static + Send + Sync,
    U: 'static + UniqueIdentifier<DataType = T>,
    CI: 'static + Read<U> + Entry<U>,
    CO: 'static + Write<U> + Size<U>,
{
    /// Creates a new logging entry for the output
    async fn log(mut self, actor: &mut Actor<CI, NO, N>) -> Self {
        match self {
            Ok(()) => panic!(r#"Input receivers have been exhausted"#),
            Err(OutputRx {
                hash,
                ref mut rxs,
                ref client,
                ..
            }) => {
                let Some(recv) = rxs.pop() else {
                    panic!(r#"Input receivers is empty"#)
                };
                (*actor.client.lock().await).entry(<CO as Size<U>>::len(&*client.lock().await));
                actor.add_input(recv, hash);
                if rxs.is_empty() {
                    Ok(())
                } else {
                    self
                }
            }
        }
    }
}
