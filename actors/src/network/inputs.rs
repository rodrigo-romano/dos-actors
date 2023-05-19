use crate::{
    Actor, ActorError, Result, UniqueIdentifier, Who,
};
use crate::interface::{self as io,Assoc, Size, Entry,Update};
use async_trait::async_trait;
use std::any::type_name;
use std::{
    collections::hash_map::DefaultHasher,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

use super::{ IntoInputs, IntoLogs, IntoLogsN, OutputRx, TryIntoInputs};

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
        Vec<flume::Receiver<io::Data<U>>>,
    )
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
    CO: 'static + Update + Send + io::Write<U>,
{
    fn legacy_into_input<CI, const N: usize>(mut self, actor: &mut Actor<CI, NO, N>) -> Self
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

impl<U, CO, const NO: usize, const NI: usize> TryIntoInputs<U, CO, NO, NI>
    for std::result::Result<(), OutputRx<U, CO, NI, NO>>
where
    Assoc<U>: Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier,
    CO: 'static + Update + Send + io::Write<U>,
{
    fn into_input<CI, const N: usize>(mut self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        CI: 'static + Update + Send + io::Read<U>,
        Self: Sized,
    {
        let Err(OutputRx{ hash, ref mut rxs,.. }) = self else { 
            panic!(r#"Input receivers have been exhausted"#) 
        };
        let Some(recv) = rxs.pop() else { panic!(r#"Input receivers is empty"#) };
        actor.add_input(recv, hash);
        if rxs.is_empty() {
            Ok(())
        } else {
            self
        }
    }
}

impl<U, CO, const NO: usize, const NI: usize> std::error::Error for OutputRx<U, CO, NI, NO>
where
    U: 'static + UniqueIdentifier + Send + Sync,
    CO: Update + io::Write<U>,
{
}
impl<U, CO, const NO: usize, const NI: usize> Display for OutputRx<U, CO, NI, NO>
where
    U: 'static + UniqueIdentifier + Send + Sync,
    CO: Update + io::Write<U>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let OutputRx { actor, output, .. } = self;
        writeln!(
            f,
            r#"TryIntoInputs for output "{}" of actor "{}", check output multiplex #"#,
            output, actor
        )
    }
}
impl<U, CO, const NO: usize, const NI: usize> Debug for OutputRx<U, CO, NI, NO>
where
    U: 'static + UniqueIdentifier + Send + Sync,
    CO: Update + io::Write<U>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(&self, f)
    }
}
#[async_trait]
impl<T, U, CI, CO, const N: usize, const NO: usize, const NI: usize> IntoLogsN<CI, N, NO>
    for std::result::Result<(), OutputRx<U, CO, NI, NO>>
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
    CI: 'static + Update + Send + io::Read<U> + Entry<U>,
    CO: 'static + Update + Send + io::Write<U>,
{
    /// Creates a new logging entry for the output
    async fn logn(mut self, actor: &mut Actor<CI, NO, N>, size: usize) -> Self {
        match self {
            Ok(()) => panic!(r#"Input receivers have been exhausted, may be {} should be multiplexed"#,type_name::<U>()) ,
            Err(OutputRx{ hash, ref mut rxs,.. }) => {
                let Some(recv) = rxs.pop() else { panic!(r#"Input receivers is empty"#) };
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
    U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
    CI: 'static + Update + Send + io::Read<U> + Entry<U>,
    CO: 'static + Update + Send + io::Write<U> + Size<U>,
{
    /// Creates a new logging entry for the output
    async fn log(mut self, actor: &mut Actor<CI, NO, N>) -> Self {
          match self {
            Ok(()) => panic!(r#"Input receivers have been exhausted"#) ,
            Err(OutputRx{ hash, ref mut rxs,ref client,.. }) => {
                let Some(recv) = rxs.pop() else { panic!(r#"Input receivers is empty"#) };
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
