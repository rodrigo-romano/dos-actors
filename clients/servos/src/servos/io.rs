use gmt_dos_actors::{
    prelude::Actor,
    system::{SystemInput, SystemOutput},
};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};

use gmt_dos_clients_m2_ctrl::positioner::AsmsPositioners;
use gmt_dos_clients_mount::Mount;

use super::GmtServoMechanisms;

// FEM inputs
impl<'a, const M1_RATE: usize, const M2_RATE: usize>
    SystemInput<DiscreteModalSolver<ExponentialMatrix>, 1, 1>
    for GmtServoMechanisms<'a, M1_RATE, M2_RATE>
{
    fn input(&mut self) -> &mut Actor<DiscreteModalSolver<ExponentialMatrix>, 1, 1> {
        &mut self.fem
    }
}
// FEM outputs
impl<'a, const M1_RATE: usize, const M2_RATE: usize>
    SystemOutput<DiscreteModalSolver<ExponentialMatrix>, 1, 1>
    for GmtServoMechanisms<'a, M1_RATE, M2_RATE>
{
    fn output(&mut self) -> &mut Actor<DiscreteModalSolver<ExponentialMatrix>, 1, 1> {
        &mut self.fem
    }
}

// Mount inputs
impl<'a, const M1_RATE: usize, const M2_RATE: usize> SystemInput<Mount<'a>, 1, 1>
    for GmtServoMechanisms<'a, M1_RATE, M2_RATE>
{
    fn input(&mut self) -> &mut Actor<Mount<'a>, 1, 1> {
        &mut self.mount
    }
}

// AsmsPositioners inputs
impl<'a, const M1_RATE: usize, const M2_RATE: usize> SystemInput<AsmsPositioners, 1, 1>
    for GmtServoMechanisms<'a, M1_RATE, M2_RATE>
{
    fn input(&mut self) -> &mut Actor<AsmsPositioners, 1, 1> {
        &mut self.m2_positioners
    }
}

// M1 inputs
impl<'a, const M1_RATE: usize, const M2_RATE: usize>
    SystemInput<gmt_dos_clients_m1_ctrl::assembly::DispatchIn, 1, 1>
    for GmtServoMechanisms<'a, M1_RATE, M2_RATE>
{
    fn input(&mut self) -> &mut Actor<gmt_dos_clients_m1_ctrl::assembly::DispatchIn, 1, 1> {
        self.m1.input()
    }
}

// M2 inputs
impl<'a, const M1_RATE: usize, const M2_RATE: usize>
    SystemInput<gmt_dos_clients_m2_ctrl::assembly::DispatchIn, 1, 1>
    for GmtServoMechanisms<'a, M1_RATE, M2_RATE>
{
    fn input(&mut self) -> &mut Actor<gmt_dos_clients_m2_ctrl::assembly::DispatchIn, 1, 1> {
        self.m2.input()
    }
}
