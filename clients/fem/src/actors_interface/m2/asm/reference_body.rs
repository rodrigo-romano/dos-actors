//! rigid body

use super::prelude::*;
use gmt_dos_clients_io::gmt_m2::asm::{M2ASMReferenceBodyForces, M2ASMReferenceBodyNodes};

/// forces
impl<S> Read<M2ASMReferenceBodyForces> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn read(&mut self, data: Data<M2ASMReferenceBodyForces>) {
        <DiscreteModalSolver<S> as Set<fem_io::MCM2RB6F>>::set(self, &data)
    }
}
/// nodes
impl<S> Write<M2ASMReferenceBodyNodes> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn write(&mut self) -> Option<Data<M2ASMReferenceBodyNodes>> {
        <DiscreteModalSolver<S> as Get<fem_io::MCM2RB6D>>::get(self).map(|data| Data::new(data))
    }
}
