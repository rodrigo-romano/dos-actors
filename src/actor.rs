use crate::{io::*, ActorError, Result, Who};
use async_trait::async_trait;
use futures::future::join_all;
use std::{fmt, ops::DerefMut, sync::Arc};
use tokio::sync::Mutex;

/// Actor client state update interface
pub trait Update {
    fn update(&mut self) {}
}

/// Builder for an actor without outputs
pub type Terminator<C, const NI: usize = 1> = Actor<C, NI, 0>;
/*
pub struct Terminator<C, const NI: usize>(PhantomData<C>);
impl<C, const NI: usize> Terminator<C, NI>
where
    C: Update + Send,
{
    /// Return an actor without outputs
    pub fn build(client: Arc<Mutex<C>>) -> Actor<C, NI, 0> {
        Actor::new(client)
    }
}
*/
/// Builder for an actor without inputs
pub type Initiator<C, const NO: usize = 1> = Actor<C, 0, NO>;
/*
pub struct Initiator<C, const NO: usize>(PhantomData<C>);
impl<C, const NO: usize> Initiator<C, NO>
where
    C: Update + Send,
{
    /// Return an actor without inputs
    pub fn build(client: Arc<Mutex<C>>) -> Actor<C, 0, NO> {
        Actor::new(client)
    }
}
 */

/// Task management abstraction
pub struct Actor<C, const NI: usize = 1, const NO: usize = 1>
where
    C: Update + Send,
{
    inputs: Option<Vec<Box<dyn InputObject>>>,
    outputs: Option<Vec<Box<dyn OutputObject>>>,
    client: Arc<Mutex<C>>,
}

impl<C, const NI: usize, const NO: usize> fmt::Display for Actor<C, NI, NO>
where
    C: Update + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.who().to_uppercase())?;
        if let Some(inputs) = self.inputs.as_ref() {
            writeln!(f, " - inputs  #{:>1}:", inputs.len())?;
            for (k, input) in self.inputs.as_ref().unwrap().iter().enumerate() {
                writeln!(f, "   {}. {}", 1 + k, (*input).who())?;
            }
        }

        if let Some(outputs) = self.outputs.as_ref() {
            writeln!(f, " - outputs #{:>1}:", outputs.len())?;
            for (k, output) in self.outputs.as_ref().unwrap().iter().enumerate() {
                writeln!(f, "   {}. {} (#{})", 1 + k, (*output).who(), output.len())?;
            }
        }

        Ok(())
    }
}
impl<C: Update + Send, const NI: usize, const NO: usize> From<C> for Actor<C, NI, NO> {
    /// Returns actor's client type name
    fn from(client: C) -> Self {
        Actor::new(Arc::new(Mutex::new(client)))
    }
}
impl<C: Update + Send, const NI: usize, const NO: usize> Who<C> for Actor<C, NI, NO> {}
impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: Update + Send,
{
    /// Creates a new empty [Actor]
    pub fn new(client: Arc<Mutex<C>>) -> Self {
        Self {
            inputs: None,
            outputs: None,
            client,
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
    pub async fn run(&mut self) {
        if let Err(e) = self.async_run().await {
            crate::print_error(format!("{} loop ended", Who::who(self)), &e);
        };
    }
}
impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: 'static + Update + Send,
{
    pub fn spawn(mut self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }
}
#[async_trait]
pub trait Run: Send {
    /// Runs the [Actor] infinite loop
    ///
    /// The loop ends when the client data is [None] or when either the sending of receiving
    /// end of a channel is dropped
    async fn async_run(&mut self) -> Result<()>;
}
#[async_trait]
impl<C, const NI: usize, const NO: usize> Run for Actor<C, NI, NO>
where
    C: Update + Send,
{
    async fn async_run(&mut self) -> Result<()> {
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
    C: 'static + Update + Send,
{
    /// Adds an output to an actor
    ///
    /// The output may be multiplexed and the same data wil be send to several inputs
    /// The default channel capacity is 1
    pub fn add_output<T, U>(
        &mut self,
        multiplex: Option<Vec<usize>>,
    ) -> (&Self, Vec<flume::Receiver<Arc<Data<T, U>>>>)
    where
        C: Write<T, U>,
        T: 'static + Send + Sync,
        U: 'static + Send + Sync,
    {
        let mut txs = vec![];
        let mut rxs = vec![];
        for &cap in &multiplex.unwrap_or(vec![1]) {
            let (tx, rx) = if cap == usize::MAX {
                flume::unbounded::<S<T, U>>()
            } else {
                flume::bounded::<S<T, U>>(cap)
            };
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
    C: 'static + Update + Send,
{
    /// Adds an output to an actor
    pub fn add_input<T, U>(&mut self, rx: flume::Receiver<Arc<Data<T, U>>>)
    where
        C: Read<T, U>,
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
    /// Bootstraps an actor outputs
    pub async fn async_bootstrap<T, U>(&mut self) -> Result<()>
    where
        T: 'static + Send + Sync,
        U: 'static + Send + Sync,
        C: Write<T, U> + Send,
    {
        if let Some(outputs) = &mut self.outputs {
            if let Some(output) = outputs
                .iter_mut()
                .find_map(|x| x.as_mut_any().downcast_mut::<Output<C, T, U, NO>>())
            {
                log::debug!("boostraping {}", Who::who(output));
                if NO >= NI {
                    output.send().await?;
                } else {
                    for _ in 0..NI / NO {
                        output.send().await?;
                    }
                }
            }
        }
        Ok(())
    }
    pub async fn bootstrap<T, U>(&mut self) -> &mut Self
    where
        T: 'static + Send + Sync,
        U: 'static + Send + Sync,
        C: Write<T, U> + Send,
    {
        if let Err(e) = self.async_bootstrap::<T, U>().await {
            crate::print_error(format!("{} distribute ended", Who::who(self)), &e);
        }
        self
    }
}
impl<C, const NI: usize, const NO: usize> Drop for Actor<C, NI, NO>
where
    C: Update + Send,
{
    fn drop(&mut self) {
        log::info!("{} dropped!", self.who());
    }
}
