//! # Subsystem
//!
//! The module implements [SubSystem] allowing to build sub-[Model]s that
//! can be inserted inside and interfaced with [Model]s.

use interface::gateway::{Gateways, WayIn, WayOut};

use crate::Actor;

use super::{Model, Unknown};

/// Interface for the sub-[Model] builder
pub trait BuildSystem<M, const NI: usize = 1, const NO: usize = 1>
where
    M: Gateways,
    <M as Gateways>::DataType: Send + Sync,
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
    <M as Gateways>::DataType: Send + Sync,
{
    fn gateway_in(&mut self) -> &mut Actor<WayIn<M>, NI, NI>;
    fn gateway_out(&mut self) -> &mut Actor<WayOut<M>, NO, NO>;
}

/// An actors sub-[Model]
pub struct SubSystem<M, const NI: usize = 1, const NO: usize = 1>
where
    M: Gateways,
    <M as Gateways>::DataType: Send + Sync,
    Model<Unknown>: From<M>,
{
    system: M,
    gateway_in: Actor<WayIn<M>, NI, NI>,
    gateway_out: Actor<WayOut<M>, NO, NO>,
}

impl<M, const NI: usize, const NO: usize> ModelGateways<M, NI, NO> for SubSystem<M, NI, NO>
where
    M: Gateways,
    <M as Gateways>::DataType: Send + Sync,
    Model<Unknown>: From<M>,
{
    fn gateway_in(&mut self) -> &mut Actor<WayIn<M>, NI, NI> {
        &mut self.gateway_in
    }

    fn gateway_out(&mut self) -> &mut Actor<WayOut<M>, NO, NO> {
        &mut self.gateway_out
    }
}

impl<M, const NI: usize, const NO: usize> From<SubSystem<M, NI, NO>> for Model<Unknown>
where
    M: Gateways + 'static,
    <M as Gateways>::DataType: Send + Sync,
    Model<Unknown>: From<M>,
{
    fn from(sys: SubSystem<M, NI, NO>) -> Self {
        sys.gateway_in + Model::<Unknown>::from(sys.system) + sys.gateway_out
    }
}

impl<M, const NI: usize, const NO: usize> SubSystem<M, NI, NO>
where
    M: Gateways + BuildSystem<M, NI, NO>,
    <M as Gateways>::DataType: Send + Sync,
    Model<Unknown>: From<M>,
{
    /// Creates a sub-system from a [Model]
    pub fn new(system: M) -> Self {
        Self {
            system,
            gateway_in: WayIn::<M>::new().into(),
            gateway_out: WayOut::<M>::new().into(),
        }
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
