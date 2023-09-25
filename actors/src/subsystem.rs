//! # Sub-systems
//!
//! The module implements [SubSystem] allowing to build sub-[Model]s that
//! can be inserted inside and interfaced with [Model]s.

use interface::UniqueIdentifier;

use crate::{
    actor::Actor,
    io::Input,
    model::{Model, Unknown},
    network::{ActorOutput, ActorOutputBuilder, AddActorInput, AddActorOutput},
};

pub mod gateway;
pub use gateway::{Gateways, WayIn, WayOut};

mod subsystem;
pub use subsystem::SubSystem;

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

impl<M, const NI: usize, const NO: usize> From<SubSystem<M, NI, NO>> for Model<Unknown>
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
}

impl<M, const NI: usize, const NO: usize> SubSystem<M, NI, NO>
where
    M: Gateways + BuildSystem<M, NI, NO>,
    Model<Unknown>: From<M>,
{
    /// Creates a sub-system from a [Model]
    pub fn new(system: M) -> Self {
        Self {
            name: None,
            flowchart: false,
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
    /// Creates the subystem flowchart
    pub fn flowchart(mut self) -> Self {
        self.flowchart = true;
        self
    }
    /// Builds the sub-[Model]
    ///
    /// Build the sub-[Model] by invoking [BuildSystem::build] on `M`
    pub fn build(mut self) -> anyhow::Result<Self> {
        self.system
            .build(&mut self.gateway_in, &mut self.gateway_out)?;
        Ok(self)
    }
}

impl<'a, M, const NI: usize, const NO: usize> AddActorOutput<'a, WayOut<M>, NO, NO>
    for SubSystem<M, NI, NO>
where
    M: Gateways + BuildSystem<M, NI, NO>,
    Model<Unknown>: From<M>,
{
    fn add_output(&'a mut self) -> ActorOutput<'a, Actor<WayOut<M>, NO, NO>> {
        ActorOutput::new(&mut self.gateway_out, ActorOutputBuilder::new(1))
    }
}

impl<U, M, const NI: usize, const NO: usize> AddActorInput<U, WayIn<M>, NI> for SubSystem<M, NI, NO>
where
    U: 'static + UniqueIdentifier<DataType = <M as Gateways>::DataType> + gateway::In,
    M: Gateways + BuildSystem<M, NI, NO> + 'static,
    Model<Unknown>: From<M>,
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
