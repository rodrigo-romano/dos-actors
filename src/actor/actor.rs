use super::plain::{PlainActor, IO};
use crate::{io::*, ActorOutputBuilder, Result, Update, Who};
use futures::future::join_all;
use std::{fmt, sync::Arc};
use tokio::sync::Mutex;

/// Actor model implementation
pub struct Actor<C, const NI: usize = 1, const NO: usize = 1>
where
    C: Update + Send,
{
    pub(super) inputs: Option<Vec<Box<dyn InputObject>>>,
    pub(crate) outputs: Option<Vec<Box<dyn OutputObject>>>,
    pub(crate) client: Arc<Mutex<C>>,
    name: Option<String>,
}

impl<C, const NI: usize, const NO: usize> From<&Actor<C, NI, NO>> for PlainActor
where
    C: Update + Send,
{
    fn from(actor: &Actor<C, NI, NO>) -> Self {
        Self {
            client: actor.name.as_ref().unwrap_or(&actor.who()).to_owned(),
            inputs_rate: NI,
            outputs_rate: NO,
            inputs: actor
                .inputs
                .as_ref()
                .map(|inputs| inputs.iter().map(|o| IO::from(o)).collect()),
            outputs: actor
                .outputs
                .as_ref()
                .map(|outputs| outputs.iter().map(|o| IO::from(o)).collect()),
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
    pub(super) async fn collect(&mut self) -> Result<&mut Self> {
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
    pub(super) async fn distribute(&mut self) -> Result<&mut Self> {
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
    pub(super) async fn bootstrap(&mut self) -> Result<&mut Self> {
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

impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: 'static + Update + Send,
{
    /// Adds a new output
    pub fn add_output(&mut self) -> (&mut Actor<C, NI, NO>, ActorOutputBuilder) {
        (self, ActorOutputBuilder::new(1))
    }
    /// Adds an input to an actor
    pub(crate) fn add_input<T, U>(&mut self, rx: flume::Receiver<Arc<Data<U>>>, hash: u64)
    where
        C: Read<U>,
        T: 'static + Send + Sync,
        U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
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
