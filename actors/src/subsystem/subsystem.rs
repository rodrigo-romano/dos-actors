use crate::{
    actor::Actor,
    model::{Model, Unknown},
};

use super::{Gateways, ModelGateways, WayIn, WayOut};

/// An actors sub-[Model]
pub struct SubSystem<M, const NI: usize = 1, const NO: usize = 1>
where
    M: Gateways,
    <M as Gateways>::DataType: Send + Sync,
    Model<Unknown>: From<M>,
{
    pub(crate) name: Option<String>,
    pub(crate) flowchart: bool,
    pub(crate) system: M,
    pub(crate) gateway_in: Actor<WayIn<M>, NI, NI>,
    pub(crate) gateway_out: Actor<WayOut<M>, NO, NO>,
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
