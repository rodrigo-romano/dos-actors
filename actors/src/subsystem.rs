//! # Sub-systems
//!
//! The module implements [SubSystem] allowing to build sub-[Model]s that
//! can be inserted inside and interfaced with [Model]s.

use std::{any::type_name, fmt::Display};

use interface::UniqueIdentifier;

use crate::{
    actor::Actor,
    io::Input,
    model::FlowChart,
    network::{ActorOutput, ActorOutputBuilder, AddActorInput, AddActorOutput},
    Check,
};

pub mod gateway;
pub use gateway::{Gateways, WayIn, WayOut};

mod subsystem;
pub use subsystem::SubSystem;

mod check;
mod flowchart;
mod task;

/**
Field selector for system of actors

Example
```
use interface::UID;
use gmt_dos_clients::{operator::Operator, Integrator};
use gmt_dos_actors::{actor::Actor, Check, subsystem::GetField};

#[derive(UID)]
pub enum Residuals {}

pub struct Controller {
    plus: Actor<operator::Operator<f64>>,
    int: Actor<Integrator<Residuals>>,
}

impl GetField for Controller {
    fn get_field(&self, idx: usize) -> Option<&dyn Check> {
        match idx {
            0 => Some(&self.plus as &dyn Check),
            1 => Some(&self.int as &dyn Check),
            _ => None,
        }
    }
}
```
*/

pub trait GetField {
    fn get_field(&self, idx: usize) -> Option<&dyn Check>;
}

/// Iterator builder for system of actors
pub struct SubSystemIterator<'a, M> {
    pub field_count: usize,
    pub system: &'a M,
}

impl<'a, M> Iterator for SubSystemIterator<'a, M>
where
    M: Gateways + GetField,
{
    type Item = &'a dyn Check;

    fn next(&mut self) -> Option<Self::Item> {
        self.field_count += 1;
        self.system.get_field(self.field_count - 1)
    }
}

trait Iter<'a, M> {
    fn iter(&'a self) -> SubSystemIterator<'a, M>;
}
impl<'a, M: Gateways> Iter<'a, M> for M {
    fn iter(&'a self) -> SubSystemIterator<'a, M> {
        SubSystemIterator {
            field_count: 0,
            system: self,
        }
    }
}

/// Interface for the sub-[Model] builder
pub trait BuildSystem<M, const NI: usize = 1, const NO: usize = 1>
where
    M: Gateways,
{
    /// Builds the model by connecting all actors
    fn build(
        &mut self,
        gateway_in: &mut Actor<WayIn<M>, NI, NI>,
        gateway_out: &mut Actor<WayOut<M>, NO, NO>,
    ) -> anyhow::Result<()>;
}
/// Interface for the gateways
pub trait ModelGateways<M, const NI: usize = 1, const NO: usize = 1>
where
    M: Gateways,
{
    fn gateway_in(&mut self) -> &mut Actor<WayIn<M>, NI, NI>;
    fn gateway_out(&mut self) -> &mut Actor<WayOut<M>, NO, NO>;
}

/* impl<M, const NI: usize, const NO: usize> From<SubSystem<M, NI, NO>> for Model<Unknown>
where
    M: Gateways + 'static,
    Model<Unknown>: From<M>,
{
    fn from(sys: SubSystem<M, NI, NO>) -> Self {
        let model = sys.gateway_in + Model::<Unknown>::from(sys.system) + sys.gateway_out;
        match (sys.name, sys.flowchart) {
            (None, true) => model.flowchart(),
            (None, false) => model,
            (Some(name), true) => model.name(name).flowchart(),
            (Some(name), false) => model.name(name),
        }
    }
} */

impl<M, const NI: usize, const NO: usize> SubSystem<M, NI, NO>
where
    M: Gateways,
{
    /// Creates a sub-system from a [Model]
    pub fn new(system: M) -> Self {
        Self {
            name: None,
            system,
            gateway_in: WayIn::<M>::new().into(),
            gateway_out: WayOut::<M>::new().into(),
        }
    }
    /// Sets the name of the subsystem
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = Some(name.to_string());
        self
    }
    /// Gets the name of the subsystem
    pub fn get_name(&self) -> String {
        self.name
            .as_ref()
            .map_or("SubSystem", |x| x.as_str())
            .into()
    }
}
impl<M, const NI: usize, const NO: usize> SubSystem<M, NI, NO>
where
    M: Gateways + 'static,
    for<'a> SubSystemIterator<'a, M>: Iterator<Item = &'a dyn Check>,
{
    /// Creates the subystem flowchart
    pub fn flowchart(self) -> Self {
        <Self as FlowChart>::flowchart(self)
    }
}
impl<M, const NI: usize, const NO: usize> SubSystem<M, NI, NO>
where
    M: Gateways + BuildSystem<M, NI, NO>,
{
    /// Builds the sub-[Model]
    ///
    /// Build the sub-[Model] by invoking [BuildSystem::build] on `M`
    pub fn build(mut self) -> anyhow::Result<Self> {
        self.system
            .build(&mut self.gateway_in, &mut self.gateway_out)?;
        Ok(self)
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

impl<M, const NI: usize, const NO: usize> Display for SubSystem<M, NI, NO>
where
    M: Gateways,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<M>())
    }
}

impl<'a, M, const NI: usize, const NO: usize> AddActorOutput<'a, WayOut<M>, NO, NO>
    for SubSystem<M, NI, NO>
where
    M: Gateways,
{
    fn add_output(&'a mut self) -> ActorOutput<'a, Actor<WayOut<M>, NO, NO>> {
        ActorOutput::new(&mut self.gateway_out, ActorOutputBuilder::new(1))
    }
}

impl<U, M, const NI: usize, const NO: usize> AddActorInput<U, WayIn<M>, NI> for SubSystem<M, NI, NO>
where
    U: 'static + UniqueIdentifier<DataType = <M as Gateways>::DataType> + gateway::In,
    M: Gateways + 'static,
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
