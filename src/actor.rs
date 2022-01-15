use crate::{io::*, ActorError, Client, Result};
use futures::future::join_all;
use std::{marker::PhantomData, ops::Deref, sync::Arc};
use tokio::sync::Mutex;

/// Builder for an actor without outputs
pub struct Terminator<C, I, const NI: usize>(PhantomData<C>, PhantomData<I>);
impl<C, I, const NI: usize> Terminator<C, I, NI>
where
    C: Client<I, I>,
    I: Default + std::fmt::Debug,
{
    /// Return an actor without outputs
    pub fn build(client: Arc<Mutex<C>>) -> Actor<C, I, I, NI, 0> {
        Actor::new(client)
    }
}

/// Builder for an actor without inputs
pub struct Initiator<C, O, const NO: usize>(PhantomData<C>, PhantomData<O>);
impl<C, O, const NO: usize> Initiator<C, O, NO>
where
    C: Client<O, O>,
    O: Default + std::fmt::Debug,
{
    /// Return an actor without inputs
    pub fn build(client: Arc<Mutex<C>>) -> Actor<C, O, O, 0, NO> {
        Actor::new(client)
    }
}

/// Task management abstraction
#[derive(Debug)]
pub struct Actor<C, I, O, const NI: usize, const NO: usize>
where
    C: Client<I, O>,
    I: Default,
    O: Default + std::fmt::Debug,
{
    pub inputs: Option<Vec<Input<I, NI>>>,
    pub outputs: Option<Vec<Output<O, NO>>>,
    client: Arc<Mutex<C>>,
}

impl<C, I, O, const NI: usize, const NO: usize> Actor<C, I, O, NI, NO>
where
    C: Client<I, O>,
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    pub fn new(client: Arc<Mutex<C>>) -> Self {
        Self {
            inputs: None,
            outputs: None,
            client,
        }
    }
    pub fn add_input(&mut self, input: Input<I, NI>) -> &mut Self {
        if let Some(inputs) = self.inputs.as_mut() {
            inputs.push(input);
        } else {
            self.inputs = Some(vec![input]);
        }
        self
    }
    pub fn add_output(&mut self, output: Output<O, NO>) -> &mut Self {
        if let Some(outputs) = self.outputs.as_mut() {
            outputs.push(output);
        } else {
            self.outputs = Some(vec![output]);
        }
        self
    }
    fn get_data(&self) -> Vec<&I> {
        self.inputs
            .as_ref()
            .unwrap()
            .iter()
            .map(|input| input.data.deref().deref())
            .collect()
    }
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
    fn disconnect(&mut self) -> &mut Self {
        println!("Dropping senders!");
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
    pub async fn task(&mut self) -> Result<()> {
        let client = self.client.clone();
        let mut client_lock = client.lock().await;
        match (self.inputs.as_ref(), self.outputs.as_ref()) {
            (Some(_), Some(_)) => {
                if NO >= NI {
                    // Decimation
                    loop {
                        for _ in 0..NO / NI {
                            self.collect().await?;
                            (*client_lock).consume(self.get_data()).update();
                        }
                        let data = (*client_lock).produce();
                        self.distribute(data).await?;
                    }
                } else {
                    // Upsampling
                    loop {
                        self.collect().await?;
                        (*client_lock).consume(self.get_data()).update();
                        for _ in 0..NI / NO {
                            let data = (*client_lock).produce();
                            self.distribute(data).await?;
                        }
                    }
                }
            }
            (None, Some(_)) => loop {
                // Initiator
                let data = (*client_lock).update().produce();
                self.distribute(data).await?;
            },
            (Some(_), None) => loop {
                // Terminator
                self.collect().await?;
                (*client_lock).consume(self.get_data()).update();
            },
            (None, None) => Ok(()),
        }
    }
}
