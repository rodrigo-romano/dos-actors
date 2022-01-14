use crate::{io::*, ActorError, Client, Result, IO};
use futures::future::join_all;
use parking_lot::Mutex;
use std::{marker::PhantomData, sync::Arc};

/// Builder for an actor without outputs
pub struct Terminator<I, const NI: usize>(PhantomData<I>);
impl<I, const NI: usize> Terminator<I, NI>
where
    I: Default + std::fmt::Debug,
{
    /// Return an actor without outputs
    pub fn new(time_idx: Arc<usize>, inputs: IO<Input<I, NI>>) -> Actor<(), I, (), NI, 0> {
        Actor {
            inputs: Some(inputs),
            outputs: None,
            time_idx,
            client: None,
        }
    }
}

/// Builder for an actor without inputs
pub struct Initiator<O, const NO: usize>(PhantomData<O>);
impl<O, const NO: usize> Initiator<O, NO>
where
    O: Default + std::fmt::Debug,
{
    /// Return an actor without inputs
    pub fn new(time_idx: Arc<usize>, outputs: IO<Output<O, NO>>) -> Actor<(), (), O, 0, NO> {
        Actor {
            inputs: None,
            outputs: Some(outputs),
            time_idx,
            client: None,
        }
    }
}

/// Task management abstraction
#[derive(Default, Debug)]
pub struct Actor<T, I, O, const NI: usize, const NO: usize>
where
    T: Client<I, O, NI, NO>,
    I: Default,
    O: Default + std::fmt::Debug,
{
    pub inputs: Option<IO<Input<I, NI>>>,
    pub outputs: Option<IO<Output<O, NO>>>,
    client: Option<Arc<Mutex<T>>>,
    time_idx: Arc<usize>,
}

impl<T, I, O, const NI: usize, const NO: usize> Actor<T, I, O, NI, NO>
where
    T: Client<I, O, NI, NO>,
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    /// Return an [Actor] with both inputs and outputs
    pub fn new(time_idx: Arc<usize>, inputs: IO<Input<I, NI>>, outputs: IO<Output<O, NO>>) -> Self {
        Self {
            inputs: Some(inputs),
            outputs: Some(outputs),
            time_idx,
            client: None,
        }
    }
    /// Gathers all the inputs from other [Actor] outputs
    pub async fn collect(&mut self) -> Result<&mut Self> {
        let futures: Vec<_> = self
            .inputs
            .as_mut()
            .ok_or(ActorError::NoInputs)?
            .iter_mut()
            .map(|input| input.recv())
            .collect();
        join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        Ok(self)
    }
    /// Sends the outputs to other [Actor] inputs
    pub async fn distribute(&mut self) -> Result<&Self> {
        let futures: Vec<_> = self
            .outputs
            .as_ref()
            .ok_or(ActorError::NoOutputs)?
            .iter()
            .map(|output| output.send())
            .collect();
        join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        Ok(self)
    }
    pub async fn task(&mut self) -> Result<()> {
        if let Some(client) = self.client.as_ref().cloned() {
            let mut client_lock = client.lock();
            match (self.inputs.as_ref(), self.outputs.as_ref()) {
                (Some(_), Some(_)) => {
                    if NO >= NI {
                        loop {
                            for _ in 0..NO / NI {
                                self.collect().await?;
                                (*client_lock)
                                    .consume(self.inputs.as_ref().unwrap())
                                    .update();
                            }
                            self.outputs = (*client_lock).produce();
                            self.distribute().await?;
                        }
                    } else {
                        loop {
                            self.collect().await?;
                            (*client_lock)
                                .consume(self.inputs.as_ref().unwrap())
                                .update();
                            for _ in 0..NI / NO {
                                self.outputs = (*client_lock).produce();
                                self.distribute().await?;
                            }
                        }
                    }
                }
                (None, Some(_)) => loop {
                    (*client_lock).update();
                    self.outputs = (*client_lock).produce();
                    self.distribute().await?;
                },
                (Some(_), None) => loop {
                    self.collect().await?;
                    (*client_lock)
                        .consume(self.inputs.as_ref().unwrap())
                        .update();
                },
                (None, None) => Ok(()),
            }
        } else {
            match (self.inputs.as_ref(), self.outputs.as_ref()) {
                (Some(_), Some(_)) => {
                    if NO >= NI {
                        loop {
                            for _ in 0..NO / NI {
                                self.collect().await?;
                            }
                            self.distribute().await?;
                        }
                    } else {
                        loop {
                            self.collect().await?;
                            for _ in 0..NI / NO {
                                self.distribute().await?;
                            }
                        }
                    }
                }
                (None, Some(_)) => loop {
                    self.distribute().await?;
                },
                (Some(_), None) => loop {
                    self.collect().await?;
                },
                (None, None) => Ok(()),
            }
        }
    }
}
