use super::plain::{PlainActor, IO};
use crate::{
    io::{Input, InputObject, OutputObject},
    network::{ActorOutput, ActorOutputBuilder, AddActorInput, AddActorOutput},
    Result,
};
use futures::{future::join_all, stream::FuturesUnordered};
use interface::{Data, Read, UniqueIdentifier, Update, Who};
use std::{
    fmt::{self, Debug},
    sync::Arc,
};
use tokio::sync::Mutex;

/// Actor model implementation
pub struct Actor<C, const NI: usize = 1, const NO: usize = 1>
where
    C: Update + Send + Sync,
{
    pub(crate) inputs: Option<Vec<Box<dyn InputObject>>>,
    pub(crate) outputs: Option<Vec<Box<dyn OutputObject>>>,
    pub(crate) client: Arc<Mutex<C>>,
    name: Option<String>,
    image: Option<String>,
}

impl<C, const NI: usize, const NO: usize> From<&Actor<C, NI, NO>> for PlainActor
where
    C: Update + Send + Sync,
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
            image: actor.image.as_ref().cloned(),
        }
    }
}

impl<C, const NI: usize, const NO: usize> fmt::Display for Actor<C, NI, NO>
where
    C: Update + Send + Sync,
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
impl<C, const NI: usize, const NO: usize> fmt::Debug for Actor<C, NI, NO>
where
    C: Update + Send + Sync + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Actor")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .field("client", &self.client)
            .field("name", &self.name)
            .field("image", &self.image)
            .finish()
    }
}

impl<C: Update + Send + Sync, const NI: usize, const NO: usize> From<C> for Actor<C, NI, NO> {
    /// Creates a new actor for the client
    fn from(client: C) -> Self {
        Actor::new(Arc::new(Mutex::new(client)))
    }
}
impl<C, S, const NI: usize, const NO: usize> From<(C, S)> for Actor<C, NI, NO>
where
    C: Update + Send + Sync,
    S: Into<String>,
{
    /// Creates a new named actor for the client
    fn from((client, name): (C, S)) -> Self {
        let mut actor = Actor::new(Arc::new(Mutex::new(client)));
        actor.name = Some(name.into());
        actor
    }
}
impl<C: Update + Send + Sync, const NI: usize, const NO: usize> Who<C> for Actor<C, NI, NO> {
    fn who(&self) -> String {
        self.name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| std::any::type_name::<C>().into())
    }
}

impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: Update + Send + Sync,
{
    /// Creates a new [Actor] for the given [client](crate::clients)
    pub fn new(client: Arc<Mutex<C>>) -> Self {
        Self {
            inputs: None,
            outputs: None,
            client,
            name: None,
            image: None,
        }
    }
    pub fn name<S: Into<String>>(self, name: S) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }
    pub fn image<S: Into<String>>(self, image: S) -> Self {
        Self {
            image: Some(image.into()),
            ..self
        }
    }
    /// Returns a pointer to the actor's client
    pub fn client(&self) -> Arc<Mutex<C>> {
        Arc::clone(&self.client)
    }
    /// Gathers all the inputs from other [Actor] outputs
    pub(super) async fn collect(&mut self) -> Result<&mut Self> {
        if let Some(inputs) = &mut self.inputs {
            let futures: FuturesUnordered<_> =
                inputs.iter_mut().map(|input| input.recv()).collect();
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
            let futures: FuturesUnordered<_> =
                outputs.iter_mut().map(|output| output.send()).collect();
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
                    .inspect(|output| {
                        interface::print_info(
                            format!("{} bootstrapped", output.highlight()),
                            None::<&dyn std::error::Error>,
                        )
                    })
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

/* impl<C, const NI: usize, const NO: usize> Actor<C, NI, NO>
where
    C: 'static + Update + Send + Sync,
{
    /// Adds a new output
    pub fn add_output(&mut self) -> ActorOutput<'_, Actor<C, NI, NO>> {
        ActorOutput::new(self, ActorOutputBuilder::new(1))
    }
} */
impl<'a, C, const NI: usize, const NO: usize> AddActorOutput<'a, C, NI, NO> for Actor<C, NI, NO>
where
    C: Update + Send + Sync + 'static,
{
    /// Adds a new output
    fn add_output(&'a mut self) -> ActorOutput<'a, Actor<C, NI, NO>> {
        ActorOutput::new(self, ActorOutputBuilder::new(1))
    }
}
impl<U, C, const NI: usize, const NO: usize> AddActorInput<U, C, NI, NO> for Actor<C, NI, NO>
where
    C: Update + Read<U> + Send + Sync + 'static,
    U: 'static + Send + Sync + UniqueIdentifier,
    <U as UniqueIdentifier>::DataType: 'static + Send + Sync,
{
    /// Adds an input to an actor
    fn add_input(&mut self, rx: flume::Receiver<Data<U>>, hash: u64) {
        let input: Input<C, <U as UniqueIdentifier>::DataType, U, NI> =
            Input::new(rx, self.client.clone(), hash);
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
