//! M2 FSM Piezo-Stack Actuators

use super::prelude::*;
use gmt_dos_clients_io::gmt_m2::fsm::{M2FSMPiezoForces, M2FSMPiezoNodes};

/// forces
impl<S> Read<M2FSMPiezoForces> for DiscreteModalSolver<S>
where
    DiscreteModalSolver<S>: Iterator,
    S: Solver + Default,
{
    fn read(&mut self, data: Data<M2FSMPiezoForces>) {
        <DiscreteModalSolver<S> as Set<fem_io::MCM2PZTF>>::set(self, &data)
    }
}
/// nodes
impl<S> Write<M2FSMPiezoNodes> for DiscreteModalSolver<S>
where
    DiscreteModalSolver<S>: Iterator,
    S: Solver + Default,
{
    fn write(&mut self) -> Option<Data<M2FSMPiezoNodes>> {
        <DiscreteModalSolver<S> as Get<fem_io::MCM2PZTD>>::get(self).map(|data| Data::new(data))
    }
}
