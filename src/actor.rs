use crate::{io::*, ActorError, Result};
use futures::future::join_all;
use std::{fmt, marker::PhantomData, ops::DerefMut, sync::Arc};
use tokio::sync::Mutex;

pub trait Updating {
    fn update(&mut self) {}
}

/// Builder for an actor without outputs
pub struct Terminator<C, const NI: usize>(PhantomData<C>);
impl<C, const NI: usize> Terminator<C, NI>
where
    C: Updating + Send,
{
    /// Return an actor without outputs
    pub fn build(client: Arc<Mutex<C>>) -> Actor<C, NI, 0> {
        Actor::new(client)
    }
}

/// Builder for an actor without inputs
pub struct Initiator<C, const NO: usize>(PhantomData<C>);
impl<C, const NO: usize> Initiator<C, NO>
where
    C: Updating + Send,
{
    /// Return an actor without inputs
    pub fn build(client: Arc<Mutex<C>>) -> Actor<C, 0, NO> {
        Actor::new(client)
    }
}

/// Task management abstraction
pub struct Actor<C, const NI: usize, const NO: usize>
where
    C: Updating + Send,
{
    pub inputs: Option<Vec<Box<dyn InputObject>>>,
    pub outputs: Option<Vec<Box<dyn OutputObject>>>,
    pub tag: Option<String>,
    pub client: Arc<Mutex<C>>,
}

impl<C, const NI: usize, const NO: usize> fmt::Display for Actor<C, NI, NO>
where
    C: Updating + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.tag.as_ref().unwrap_or(&"Actor".to_string()))?;
        if let Some(inputs) = self.inputs.as_ref() {
            writeln!(f, " - inputs  #{:>1}", inputs.len())?;
        }

        if let Some(outputs) = self.outputs.as_ref() {
            writeln!(f, " - outputs #{:>1}", outputs.len(),)?
        }

        Ok(())
    }
}

impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: Updating + Send,
{
    /// Creates a new empty [Actor]
    pub fn new(client: Arc<Mutex<C>>) -> Self {
        Self {
            inputs: None,
            outputs: None,
            tag: None,
            client,
        }
    }
    /// Tags the actor
    pub fn tag<S: Into<String>>(self, tag: S) -> Self {
        Self {
            tag: Some(tag.into()),
            ..self
        }
    }
    /// Gathers all the inputs from other [Actor] outputs
    pub async fn collect(&mut self) -> Result<()> {
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
        Ok(())
    }
    /// Sends the outputs to other [Actor] inputs
    pub async fn distribute(&mut self) -> Result<&Self> {
        let futures: Vec<_> = self
            .outputs
            .as_mut()
            .ok_or(ActorError::NoOutputs)?
            .iter_mut()
            .map(|output| output.send())
            .collect();
        join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        Ok(self)
    }
    /// Runs the [Actor] infinite loop
    ///
    /// The loop ends when the client data is [None] or when either the sending of receiving
    /// end of a channel is dropped
    pub async fn run(&mut self) -> Result<()> {
        //let client_clone = self.client.clone();
        //let mut client_lock = client_clone.lock().await;
        //let client = client_lock.deref_mut();
        match (self.inputs.as_ref(), self.outputs.as_ref()) {
            (Some(_), Some(_)) => {
                if NO >= NI {
                    // Decimation
                    loop {
                        for _ in 0..NO / NI {
                            self.collect().await?;
                            self.client.lock().await.deref_mut().update();
                        }
                        self.distribute().await?;
                    }
                } else {
                    // Upsampling
                    loop {
                        self.collect().await?;
                        self.client.lock().await.deref_mut().update();
                        for _ in 0..NI / NO {
                            self.distribute().await?;
                        }
                    }
                }
            }
            (None, Some(_)) => loop {
                // Initiator
                self.client.lock().await.deref_mut().update();
                self.distribute().await?;
            },
            (Some(_), None) => loop {
                // Terminator
                match self.collect().await {
                    Ok(_) => {
                        self.client.lock().await.deref_mut().update();
                    }
                    Err(e) => break Err(e),
                }
            },
            (None, None) => Ok(()),
        }
    }
}
impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: 'static + Updating + Send,
{
    /// Adds an output to an actor
    ///
    /// The output may be multiplexed and the same data wil be send to several inputs
    pub fn add_output<T, U>(
        &mut self,
        multiplex: Option<usize>,
    ) -> (&Self, Vec<flume::Receiver<Arc<Data<T, U>>>>)
    where
        C: Producing<T, U>,
        T: 'static + Send + Sync + fmt::Debug,
        U: 'static + Send + Sync + fmt::Debug,
    {
        let mut txs = vec![];
        let mut rxs = vec![];
        for _ in 0..multiplex.unwrap_or(1) {
            let (tx, rx) = flume::bounded::<S<T, U>>(1);
            txs.push(tx);
            rxs.push(rx);
        }
        let output: Output<C, T, U, NO> = Output::new(txs, self.client.clone());
        if let Some(ref mut outputs) = self.outputs {
            outputs.push(Box::new(output));
        } else {
            self.outputs = Some(vec![Box::new(output)]);
        }
        (self, rxs)
    }
}
impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: 'static + Updating + Send,
{
    /// Adds an output to an actor
    pub fn add_input<T, U>(&mut self, rx: flume::Receiver<Arc<Data<T, U>>>)
    where
        C: Consuming<T, U>,
        T: 'static + Send + Sync,
        U: 'static + Send + Sync,
    {
        let input: Input<C, T, U, NI> = Input::new(rx, self.client.clone());
        if let Some(ref mut inputs) = self.inputs {
            inputs.push(Box::new(input));
        } else {
            self.inputs = Some(vec![Box::new(input)]);
        }
    }
}
impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: Updating + Send,
{
    /// Bootstraps an actor outputs
    pub async fn bootstrap(&mut self) -> Result<&mut Self> {
        if NO >= NI {
            self.distribute().await?;
        } else {
            for _ in 0..NI / NO {
                self.distribute().await?;
            }
        }
        Ok(self)
    }
}
