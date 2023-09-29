//! # Actors subsystem
//!
//! The module implements [SubSystem] allowing to build sub-[Model]s that
//! can be inserted inside and interfaced with [Model]s.
//!
//! [Model]: crate::model::Model

use crate::{actor::Actor, framework::model::Check};

pub mod gateway;
pub use gateway::{Gateways, WayIn, WayOut};

mod subsystem;
pub use subsystem::{Built, SubSystem};

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
    plus: Actor<Operator<f64>>,
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

/// Interface for the sub-[Model](crate::model::Model) builder
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
