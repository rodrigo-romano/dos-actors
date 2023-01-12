use super::{
    plain::{PlainActor, PlainIO, PlainOutput},
    Task, Update,
};
use crate::{io::*, ActorError, ActorOutputBuilder, Result, Who};
use async_trait::async_trait;
use futures::future::join_all;
use std::{fmt, sync::Arc};
use tokio::sync::Mutex;

/// Actor model implementation
pub struct Actor<C, const NI: usize = 1, const NO: usize = 1>
where
    C: Update + Send,
{
    inputs: Option<Vec<Box<dyn InputObject>>>,
    pub(crate) outputs: Option<Vec<Box<dyn OutputObject>>>,
    pub(crate) client: Arc<Mutex<C>>,
    name: Option<String>,
}

impl<C, const NI: usize, const NO: usize> From<&Actor<C, NI, NO>> for PlainActor
where
    C: Update + Send,
{
    fn from(actor: &Actor<C, NI, NO>) -> Self {
        use PlainOutput::*;
        Self {
            client: actor.name.as_ref().unwrap_or(&actor.who()).to_owned(),
            inputs_rate: NI,
            outputs_rate: NO,
            inputs: actor.inputs.as_ref().map(|inputs| {
                inputs
                    .iter()
                    .map(|o| PlainIO::new(o.who(), o.get_hash()))
                    .collect()
            }),
            outputs: actor.outputs.as_ref().map(|outputs| {
                outputs
                    .iter()
                    .map(|o| {
                        if o.bootstrap() {
                            Bootstrap(PlainIO::new(o.who(), o.get_hash()))
                        } else {
                            Regular(PlainIO::new(o.who(), o.get_hash()))
                        }
                    })
                    .collect()
            }),
            hash: 0,
        }
    }
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
                writeln!(f, "   {}. {}", 1 + k, input)?;
            }
        }

        if let Some(outputs) = self.outputs.as_ref() {
            writeln!(f, " - outputs #{:>1}:", outputs.len())?;
            for (k, output) in self.outputs.as_ref().unwrap().iter().enumerate() {
                /*                     writeln!(
                    f,
                    "   {}. {} (#{}, bootstrap)",
                    1 + k,
                    (*output).who(),
                    output.len()
                )?; */
                writeln!(f, "   {}. {}", 1 + k, output)?;
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
impl<C, S, const NI: usize, const NO: usize> From<(C, S)> for Actor<C, NI, NO>
where
    C: Update + Send,
    S: Into<String>,
{
    /// Creates a new named actor for the client
    fn from((client, name): (C, S)) -> Self {
        let mut actor = Actor::new(Arc::new(Mutex::new(client)));
        actor.name = Some(name.into());
        actor
    }
}
impl<C: Update + Send, const NI: usize, const NO: usize> Who<C> for Actor<C, NI, NO> {
    fn who(&self) -> String {
        self.name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| std::any::type_name::<C>().into())
    }
}

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
            name: None,
        }
    }
    pub fn name<S: Into<String>>(self, name: S) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }
    /// Gathers all the inputs from other [Actor] outputs
    async fn collect(&mut self) -> Result<&mut Self> {
        if let Some(inputs) = &mut self.inputs {
            let futures: Vec<_> = inputs.iter_mut().map(|input| input.recv()).collect();
            join_all(futures)
                .await
                .into_iter()
                .collect::<Result<Vec<_>>>()?;
        }
        Ok(self)
    }
    /// Sends the outputs to other [Actor] inputs
    async fn distribute(&mut self) -> Result<&mut Self> {
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
                            self.collect().await?.client.lock().await.update();
                        }
                        self.distribute().await?;
                    }
                } else {
                    // Upsampling
                    loop {
                        self.collect().await?.client.lock().await.update();
                        for _ in 0..NI / NO {
                            self.distribute().await?;
                        }
                    }
                }
            }
            (None, Some(_)) => loop {
                // Initiator
                self.client.lock().await.update();
                self.distribute().await?;
            },
            (Some(_), None) => loop {
                // Terminator
                self.collect().await?.client.lock().await.update();
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
    fn n_inputs(&self) -> usize {
        self.inputs.as_ref().map_or(0, |i| i.len())
    }
    fn n_outputs(&self) -> usize {
        self.outputs.as_ref().map_or(0, |o| o.iter().map(|o| o.len()).sum())
    }
    fn inputs_hashes(&self) -> Vec<u64> {
        self.inputs.as_ref().map_or(Vec::new(), |inputs| {
            inputs.iter().map(|input| input.get_hash()).collect()
        })
    }
    fn outputs_hashes(&self) -> Vec<u64> {
        self.outputs.as_ref().map_or(Vec::new(), |outputs| {
            outputs
                .iter()
                .flat_map(|output| vec![output.get_hash(); output.len()])
                .collect()
        })
    }
    fn as_plain(&self) -> PlainActor {
        self.into()
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
    /// Adds an output to an actor
    pub(crate) fn add_input<T, U>(&mut self, rx: flume::Receiver<Arc<Data<U>>>, hash: u64)
    where
        C: Read<U>,
        T: 'static + Send + Sync,
        U: 'static + Send + Sync + UniqueIdentifier<Data = T>,
    {
        let input: Input<C, T, U, NI> = Input::new(rx, self.client.clone(), hash);
        if let Some(ref mut inputs) = self.inputs {
            inputs.push(Box::new(input));
        } else {
            self.inputs = Some(vec![Box::new(input)]);
        }
    }
}
/*
impl<C, const NI: usize, const NO: usize> Drop for Actor<C, NI, NO>
where
    C: Update + Send,
{
    fn drop(&mut self) {
        log::info!("{} dropped!", self.who());
    }
}
*/
