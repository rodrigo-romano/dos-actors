use crate::{io::*, ActorError, Client, Result};
use futures::future::join_all;
use std::{marker::PhantomData, ops::Deref, sync::Arc};

/// Builder for an actor without outputs
pub struct Terminator<I, const NI: usize>(PhantomData<I>);
impl<I, const NI: usize> Terminator<I, NI>
where
    I: Default + std::fmt::Debug,
{
    /// Return an actor without outputs
    pub fn build() -> Actor<I, (), NI, 0> {
        Actor::new()
    }
}

/// Builder for an actor without inputs
pub struct Initiator<O, const NO: usize>(PhantomData<O>);
impl<O, const NO: usize> Initiator<O, NO>
where
    O: Default + std::fmt::Debug,
{
    /// Return an actor without inputs
    pub fn build() -> Actor<(), O, 0, NO> {
        Actor::new()
    }
}

/// Task management abstraction
#[derive(Debug)]
pub struct Actor<I, O, const NI: usize, const NO: usize>
where
    I: Default,
    O: Default + std::fmt::Debug,
{
    pub inputs: Option<Vec<Input<I, NI>>>,
    pub outputs: Option<Vec<Output<O, NO>>>,
}

impl<I, O, const NI: usize, const NO: usize> Actor<I, O, NI, NO>
where
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    /// Creates a new empty [Actor]
    pub fn new() -> Self {
        Self {
            inputs: None,
            outputs: None,
        }
    }
    /// Adds an input to [Actor]
    pub fn add_input(&mut self, input: Input<I, NI>) -> &mut Self {
        if let Some(inputs) = self.inputs.as_mut() {
            inputs.push(input);
        } else {
            self.inputs = Some(vec![input]);
        }
        self
    }
    /// Adds an output to [Actor]
    pub fn add_output(&mut self, output: Output<O, NO>) -> &mut Self {
        if let Some(outputs) = self.outputs.as_mut() {
            outputs.push(output);
        } else {
            self.outputs = Some(vec![output]);
        }
        self
    }
    // Gathers the [Actor::inputs] data
    fn get_data(&self) -> Vec<&I> {
        self.inputs
            .as_ref()
            .unwrap()
            .iter()
            .map(|input| input.data.deref().deref())
            .collect()
    }
    // Allocates new data to the [Actor::outputs]
    fn set_data(&mut self, new_data: Vec<O>) -> &mut Self {
        self.outputs
            .as_mut()
            .unwrap()
            .iter_mut()
            .zip(new_data.into_iter())
            .for_each(|(output, data)| {
                output.data = Arc::new(Data(data));
            });
        self
    }
    // Drops all [Actor::outputs] senders
    fn disconnect(&mut self) -> &mut Self {
        self.outputs
            .as_mut()
            .unwrap()
            .iter_mut()
            .for_each(|output| output.tx.iter_mut().for_each(drop));
        self
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
    pub async fn distribute(&mut self, data: Option<Vec<O>>) -> Result<&Self> {
        if let Some(data) = data {
            self.set_data(data);
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
        } else {
            self.disconnect();
            Err(ActorError::Disconnected)
        }
    }
    /// Runs the [Actor] infinite loop
    ///
    /// The loop ends when the client data is [None] or when either the sending of receiving
    /// end of the channel is dropped
    pub async fn run<C: Client<I = I, O = O>>(&mut self, client: &mut C) -> Result<()> {
        match (self.inputs.as_ref(), self.outputs.as_ref()) {
            (Some(_), Some(_)) => {
                if NO >= NI {
                    // Decimation
                    loop {
                        for _ in 0..NO / NI {
                            self.collect().await?;
                            client.consume(self.get_data()).update();
                        }
                        let data = client.produce();
                        self.distribute(data).await?;
                    }
                } else {
                    // Upsampling
                    loop {
                        self.collect().await?;
                        client.consume(self.get_data()).update();
                        for _ in 0..NI / NO {
                            let data = client.produce();
                            self.distribute(data).await?;
                        }
                    }
                }
            }
            (None, Some(_)) => loop {
                // Initiator
                let data = client.update().produce();
                self.distribute(data).await?;
            },
            (Some(_), None) => loop {
                // Terminator
                match self.collect().await {
                    Ok(_) => {
                        client.consume(self.get_data()).update();
                    }
                    Err(e) => break Err(e),
                }
            },
            (None, None) => Ok(()),
        }
    }
}
