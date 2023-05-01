//! M2 rigid body motions

use super::prelude::*;
use gmt_dos_clients_io::gmt_m2::M2RigidBodyMotions;

impl<S> Size<M2RigidBodyMotions> for DiscreteModalSolver<S>
where
    DiscreteModalSolver<S>: Iterator,
    S: Solver + Default,
{
    fn len(&self) -> usize {
        42
    }
}
#[cfg(not(feature = "mcm2lcl"))]
impl<S> Write<M2RigidBodyMotions> for DiscreteModalSolver<S>
where
    S: Solver + Default,
{
    fn write(&mut self) -> Option<Data<M2RigidBodyMotions>> {
        <DiscreteModalSolver<S> as Get<fem_io::MCM2Lcl6D>>::get(self).map(|data| Data::new(data))
    }
}
#[cfg(feature = "mcm2lcl")]
impl<S> Write<M2RigidBodyMotions> for DiscreteModalSolver<S>
where
    S: Solver + Default,
{
    fn write(&mut self) -> Option<Data<M2RigidBodyMotions>> {
        <DiscreteModalSolver<S> as Get<fem_io::MCM2Lcl>>::get(self).map(|data| Data::new(data))
    }
}
