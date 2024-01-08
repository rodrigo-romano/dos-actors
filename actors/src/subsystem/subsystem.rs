use std::{any::type_name, fmt::Display, marker::PhantomData};

use interface::UniqueIdentifier;

use crate::{
    actor::io::Input,
    actor::Actor,
    framework::model::{Check, FlowChart},
    framework::network::{ActorOutput, ActorOutputBuilder, AddActorInput, AddActorOutput},
};

use super::{gateway, BuildSystem, Gateways, ModelGateways, SubSystemIterator, WayIn, WayOut};

#[derive(Clone)]
pub enum New {}
#[derive(Clone)]
pub enum Built {}

pub trait State {}
impl State for New {}
impl State for Built {}

/// An actors sub-[Model](crate::model::Model)
pub struct SubSystem<M, const NI: usize = 1, const NO: usize = 1, S = New>
where
    M: Gateways + Clone,
{
    pub(crate) name: Option<String>,
    pub(crate) system: M,
    pub(crate) gateway_in: Actor<WayIn<M>, NI, NI>,
    pub(crate) gateway_out: Actor<WayOut<M>, NO, NO>,
    state: PhantomData<S>,
}

impl<M, const NI: usize, const NO: usize> Clone for SubSystem<M, NI, NO, Built>
where
    M: Gateways + Clone + BuildSystem<M, NI, NO>,
{
    fn clone(&self) -> Self {
        let this = Self {
            name: self.name.clone(),
            system: self.system.clone(),
            gateway_in: self.gateway_in.clone(),
            gateway_out: self.gateway_out.clone(),
            state: self.state.clone(),
        };
        let Self {
            name,
            mut system,
            mut gateway_in,
            mut gateway_out,
            ..
        } = this;
        system.build(&mut gateway_in, &mut gateway_out).unwrap();
        SubSystem {
            name,
            system,
            gateway_in,
            gateway_out,
            state: PhantomData,
        }
    }
}

unsafe impl<M, const NI: usize, const NO: usize, S> Send for SubSystem<M, NI, NO, S> where
    M: Gateways + Clone
{
}
unsafe impl<M, const NI: usize, const NO: usize, S> Sync for SubSystem<M, NI, NO, S> where
    M: Gateways + Clone
{
}

impl<M, const NI: usize, const NO: usize> SubSystem<M, NI, NO>
where
    M: Gateways + Clone,
{
    /// Creates a sub-system from a [Model](crate::model::Model)
    pub fn new(system: M) -> Self {
        Self {
            name: None,
            system,
            gateway_in: WayIn::<M>::new().into(),
            gateway_out: WayOut::<M>::new().into(),
            state: PhantomData,
        }
    }
    /// Sets the name of the subsystem
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }
}
/* impl<M, S, const NI: usize, const NO: usize> SubSystem<M, NI, NO, S>
where
    M: Gateways + Clone,
    S: State,
{
    /// Gets the name of the subsystem
    pub fn get_name(&self) -> String {
        self.name
            .as_ref()
            .map_or("SubSystem", |x| x.as_str())
            .into()
    }
} */

impl<M, const NI: usize, const NO: usize> SubSystem<M, NI, NO, Built>
where
    M: Gateways + Clone + 'static,
    for<'a> SubSystemIterator<'a, M>: Iterator<Item = &'a dyn Check>,
{
    /// Creates the subystem flowchart
    pub fn flowchart(self) -> Self {
        <Self as FlowChart>::flowchart(self)
    }
}

impl<M, const NI: usize, const NO: usize> SubSystem<M, NI, NO, New>
where
    M: Gateways + Clone + BuildSystem<M, NI, NO>,
{
    /// Builds the sub-[Model](crate::model::Model)
    ///
    /// Build the sub-[Model](crate::model::Model) by invoking [BuildSystem::build] on `M`
    pub fn build(self) -> anyhow::Result<SubSystem<M, NI, NO, Built>> {
        let Self {
            name,
            mut system,
            mut gateway_in,
            mut gateway_out,
            ..
        } = self;
        system.build(&mut gateway_in, &mut gateway_out)?;
        Ok(SubSystem {
            name,
            system,
            gateway_in,
            gateway_out,
            state: PhantomData,
        })
    }

    /*     async fn bootstrap_gateways(
        outputs: &mut Vec<Box<dyn OutputObject>>,
        inputs: &mut Vec<Box<dyn InputObject>>,
    ) -> std::result::Result<(), ActorError> {
        let futures: Vec<_> = outputs
            .iter_mut()
            .zip(inputs.iter_mut())
            .filter(|(output, _input)| output.bootstrap())
            /*             .inspect(|(output, _input)| {
                println!(
                    "{}/{:?}",
                    format!("{} bootstrapped", output.highlight()),
                    None::<&dyn std::error::Error>,
                )
            }) */
            .map(|(output, input)| async {
                input.recv().await?;
                output.send().await?;
                Ok(())
            })
            .collect();
        join_all(futures)
            .await
            .into_iter()
            .collect::<std::result::Result<Vec<_>, ActorError>>()?;
        Ok(())
    } */

    /*     async fn bootstrap_in(&mut self) -> std::result::Result<&mut Self, ActorError> {
        if let (Some(outputs), Some(inputs)) =
            (&mut self.gateway_in.outputs, &mut self.gateway_in.inputs)
        {
            Self::bootstrap_gateways(outputs, inputs).await?;
        }
        Ok(self)
    }

    async fn bootstrap_out(&mut self) -> std::result::Result<&mut Self, ActorError> {
        if let (Some(outputs), Some(inputs)) =
            (&mut self.gateway_out.outputs, &mut self.gateway_out.inputs)
        {
            Self::bootstrap_gateways(outputs, inputs).await?;
        }
        Ok(self)
    } */
}

/// [ModelGateways] implementation
impl<M, const NI: usize, const NO: usize> ModelGateways<M, NI, NO> for SubSystem<M, NI, NO>
where
    M: Gateways + Clone,
{
    fn gateway_in(&mut self) -> &mut Actor<WayIn<M>, NI, NI> {
        &mut self.gateway_in
    }

    fn gateway_out(&mut self) -> &mut Actor<WayOut<M>, NO, NO> {
        &mut self.gateway_out
    }
}

impl<M, S, const NI: usize, const NO: usize> Display for SubSystem<M, NI, NO, S>
where
    M: Gateways + Clone,
    S: State,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<M>())
    }
}

impl<'a, M, const NI: usize, const NO: usize> AddActorOutput<'a, WayOut<M>, NO, NO>
    for SubSystem<M, NI, NO, Built>
where
    M: Gateways + Clone,
{
    fn add_output(&'a mut self) -> ActorOutput<'a, Actor<WayOut<M>, NO, NO>> {
        ActorOutput::new(&mut self.gateway_out, ActorOutputBuilder::new(1))
    }
}

impl<U, M, const NI: usize, const NO: usize> AddActorInput<U, WayIn<M>, NI, NO>
    for SubSystem<M, NI, NO, Built>
where
    U: 'static + UniqueIdentifier<DataType = <M as Gateways>::DataType> + gateway::In,
    M: Gateways + Clone + 'static,
{
    fn add_input(&mut self, rx: flume::Receiver<interface::Data<U>>, hash: u64) {
        let input: Input<WayIn<M>, U, NI> = Input::new(rx, self.gateway_in.client.clone(), hash);
        if let Some(ref mut inputs) = self.gateway_in.inputs {
            inputs.push(Box::new(input));
        } else {
            self.gateway_in.inputs = Some(vec![Box::new(input)]);
        }
    }
}
