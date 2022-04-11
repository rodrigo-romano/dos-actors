use super::{Task, Update};
use crate::{io::*, ActorError, ActorOutputBuilder, Result, Who};
use async_trait::async_trait;
use futures::future::join_all;
use std::{fmt, ops::DerefMut, sync::Arc};
use tokio::sync::Mutex;

/// Actor model implementation
pub struct Actor<C, const NI: usize = 1, const NO: usize = 1>
where
    C: Update + Send,
{
    inputs: Option<Vec<Box<dyn InputObject>>>,
    pub(crate) outputs: Option<Vec<Box<dyn OutputObject>>>,
    pub(crate) client: Arc<Mutex<C>>,
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
    /// Creates a new actor for the client
    fn from(client: C) -> Self {
        Actor::new(Arc::new(Mutex::new(client)))
    }
}
impl<C: Update + Send, const NI: usize, const NO: usize> Who<C> for Actor<C, NI, NO> {}

impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: Update + Send,
{
    /// Creates a new [Actor] for the given [client](crate::clients)
    pub fn new(client: Arc<Mutex<C>>) -> Self {
        Self {
            inputs: None,
            outputs: None,
            client,
        }
    }
    /// Gathers all the inputs from other [Actor] outputs
    async fn collect(&mut self) -> Result<()> {
        if let Some(inputs) = &mut self.inputs {
            let futures: Vec<_> = inputs.iter_mut().map(|input| input.recv()).collect();
            join_all(futures)
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()?;
        }
        Ok(())
    }
    /// Sends the outputs to other [Actor] inputs
    async fn distribute(&mut self) -> Result<&Self> {
        if let Some(outputs) = &mut self.outputs {
            let futures: Vec<_> = outputs.iter_mut().map(|output| output.send()).collect();
            join_all(futures)
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()?;
        }
        Ok(self)
    }
    /// Invokes outputs senders
    async fn bootstrap(&mut self) -> Result<&mut Self> {
        if let Some(outputs) = &mut self.outputs {
            async fn inner(outputs: &mut Vec<Box<dyn OutputObject>>) -> Result<()> {
                let futures: Vec<_> = outputs
                    .iter_mut()
                    .filter(|output| output.bootstrap())
                    .map(|output| output.send())
                    .collect();
                join_all(futures)
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>>>()?;
                Ok(())
            }
            if NO >= NI {
                inner(outputs).await?;
            } else {
                for _ in 0..NI / NO {
                    inner(outputs).await?;
                }
            }
        }
        Ok(self)
    }
}

#[async_trait]
impl<C, const NI: usize, const NO: usize> Task for Actor<C, NI, NO>
where
    C: 'static + Update + Send,
{
    /// Run the actor loop in a dedicated thread
    fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.task().await;
        })
    }
    /// Run the actor loop
    async fn task(&mut self) {
        match self.bootstrap().await {
            Err(e) => crate::print_error(format!("{} bootstrapping failed", Who::who(self)), &e),
            Ok(_) => {
                if let Err(e) = self.async_run().await {
                    crate::print_error(format!("{} loop ended", Who::who(self)), &e);
                }
            }
        }
    }
    /// Starts the actor infinite loop
    async fn async_run(&mut self) -> Result<()> {
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
    fn check_inputs(&self) -> Result<()> {
        match self.inputs {
            Some(_) if NI == 0 => Err(ActorError::SomeInputsZeroRate(Who::who(self))),
            None if NI > 0 => Err(ActorError::NoInputsPositiveRate(Who::who(self))),
            _ => Ok(()),
        }
    }
    fn check_outputs(&self) -> Result<()> {
        match self.outputs {
            Some(_) if NO == 0 => Err(ActorError::SomeOutputsZeroRate(Who::who(self))),
            None if NO > 0 => Err(ActorError::NoOutputsPositiveRate(Who::who(self))),
            _ => Ok(()),
        }
    }
    fn client_typename(&self) -> String {
        self.who()
    }
    fn outputs_typename(&self) -> Option<Vec<String>> {
        self.outputs
            .as_ref()
            .map(|outputs| outputs.iter().map(|o| o.who()).collect())
    }
    fn inputs_typename(&self) -> Option<Vec<String>> {
        self.inputs
            .as_ref()
            .map(|inputs| inputs.iter().map(|o| o.who()).collect())
    }
}

impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: 'static + Update + Send,
{
    /// Adds a new output
    pub fn add_output(&mut self) -> (&mut Actor<C, NI, NO>, ActorOutputBuilder) {
        (self, ActorOutputBuilder::new(1))
    }
    /*
        /// Adds an output to an actor
        ///
        /// The output may be multiplexed and the same data wil be send to several inputs
        /// The default channel capacity is 1
        fn add_output<T, U>(
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
            for &cap in &multiplex.unwrap_or_else(|| vec![1]) {
                let (tx, rx) = if cap == usize::MAX {
                    flume::unbounded::<S<T, U>>()
                } else {
                    flume::bounded::<S<T, U>>(cap)
                };
                txs.push(tx);
                rxs.push(rx);
            }
            let output: Output<C, T, U, NO> = Output::builder(self.client.clone()).senders(txs).build();
            if let Some(ref mut outputs) = self.outputs {
                outputs.push(Box::new(output));
            } else {
                self.outputs = Some(vec![Box::new(output)]);
            }
            (self, rxs)
        }
    */
}
impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: 'static + Update + Send,
{
    /// Adds an output to an actor
    pub(crate) fn add_input<T, U>(&mut self, rx: flume::Receiver<Arc<Data<T, U>>>)
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
    /*
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
    */
}
impl<C, const NI: usize, const NO: usize> Drop for Actor<C, NI, NO>
where
    C: Update + Send,
{
    fn drop(&mut self) {
        log::info!("{} dropped!", self.who());
    }
}
