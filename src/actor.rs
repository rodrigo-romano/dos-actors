use crate::{io::*, ActorError, Result};
use futures::future::join_all;
use std::{marker::PhantomData, sync::Arc};

/// Builder for an actor without outputs
pub struct Terminator<I, const NI: usize>(PhantomData<I>);
impl<I, const NI: usize> Terminator<I, NI>
where
    I: Default + std::fmt::Debug,
{
    /// Return an actor without outputs
    pub fn new(time_idx: Arc<usize>, inputs: IO<Input<I, NI>>) -> Actor<I, (), NI, 0> {
        Actor {
            inputs: Some(inputs),
            outputs: None,
            time_idx,
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
    pub fn new(time_idx: Arc<usize>, outputs: IO<Output<O, NO>>) -> Actor<(), O, 0, NO> {
        Actor {
            inputs: None,
            outputs: Some(outputs),
            time_idx,
        }
    }
}

type IO<S> = Vec<S>;
/// Task management abstraction
#[derive(Default, Debug)]
pub struct Actor<I, O, const NI: usize, const NO: usize>
where
    I: Default,
    O: Default + std::fmt::Debug,
{
    pub inputs: Option<IO<Input<I, NI>>>,
    pub outputs: Option<IO<Output<O, NO>>>,
    time_idx: Arc<usize>,
}

impl<I, O, const NI: usize, const NO: usize> Actor<I, O, NI, NO>
where
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    /// Return an [Actor] with both inputs and outputs
    pub fn new(time_idx: Arc<usize>, inputs: IO<Input<I, NI>>, outputs: IO<Output<O, NO>>) -> Self {
        Self {
            inputs: Some(inputs),
            outputs: Some(outputs),
            time_idx,
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
    pub async fn distribute(&self) -> Result<&Self> {
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
    pub fn compute(&mut self) -> Result<&mut Self> {
        Ok(self)
    }
    pub async fn task(&mut self) -> Result<()> {
        match (self.inputs.as_ref(), self.outputs.as_ref()) {
            (Some(_), Some(_)) => {
                if NO >= NI {
                    loop {
                        for _ in 0..NO / NI {
                            self.collect().await?.compute()?;
                        }
                        self.distribute().await?;
                    }
                } else {
                    loop {
                        self.collect().await?.compute()?;
                        for _ in 0..NI / NO {
                            self.distribute().await?;
                        }
                    }
                }
            }
            (None, Some(_)) => loop {
                self.compute()?.distribute().await?;
            },
            (Some(_), None) => loop {
                self.collect().await?.compute()?;
            },
            (None, None) => Ok(()),
        }
    }
}
