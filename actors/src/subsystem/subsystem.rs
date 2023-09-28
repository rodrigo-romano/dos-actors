use crate::actor::Actor;

use super::{Gateways, ModelGateways, WayIn, WayOut};

/// An actors sub-[Model](crate::model::Model)
pub struct SubSystem<M, const NI: usize = 1, const NO: usize = 1>
where
    M: Gateways,
{
    pub(crate) name: Option<String>,
    pub(crate) system: M,
    pub(crate) gateway_in: Actor<WayIn<M>, NI, NI>,
    pub(crate) gateway_out: Actor<WayOut<M>, NO, NO>,
}

unsafe impl<M, const NI: usize, const NO: usize> Send for SubSystem<M, NI, NO> where M: Gateways {}
unsafe impl<M, const NI: usize, const NO: usize> Sync for SubSystem<M, NI, NO> where M: Gateways {}

impl<M, const NI: usize, const NO: usize> ModelGateways<M, NI, NO> for SubSystem<M, NI, NO>
where
    M: Gateways,
{
    fn gateway_in(&mut self) -> &mut Actor<WayIn<M>, NI, NI> {
        &mut self.gateway_in
    }

    fn gateway_out(&mut self) -> &mut Actor<WayOut<M>, NO, NO> {
        &mut self.gateway_out
    }
}
