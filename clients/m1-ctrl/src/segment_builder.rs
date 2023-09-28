use crate::subsystems::SegmentControl;
use crate::{Calibration, Segment};
use gmt_dos_actors::prelude::*;
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m1::segment::{
    ActuatorAppliedForces, ActuatorCommandForces, HardpointsForces, HardpointsMotion, RBM,
};
use interface::{Update, Write};

/// Buider for M1 segment control system
///
/// The control system is made of the [Actuators], [Hardpoints] and [LoadCells] controllers.
pub struct SegmentBuilder<
    'a,
    const ID: u8,
    const ACTUATOR_RATE: usize,
    Crbm,
    Cactuator,
    const N_RBM: usize,
    const N_ACTUATOR: usize,
> where
    Crbm: Update + Write<RBM<ID>> + Send + Sync + 'static,
    Cactuator: Update + Write<ActuatorCommandForces<ID>> + Send + Sync + 'static,
{
    rbm_setpoint_actor: &'a mut Actor<Crbm, N_RBM, 1>,
    actuator_setpoint_actor: &'a mut Actor<Cactuator, N_ACTUATOR, 1>,
    calibration: Calibration,
}

impl<'a, const ID: u8, const ACTUATOR_RATE: usize> Segment<ID, ACTUATOR_RATE> {
    pub fn builder<Crbm, Cactuator, const N_ACTUATOR: usize, const N_RBM: usize>(
        calibration: Calibration,
        rbm_setpoint_actor: &'a mut Actor<Crbm, N_RBM, 1>,
        actuator_setpoint_actor: &'a mut Actor<Cactuator, N_ACTUATOR, 1>,
    ) -> SegmentBuilder<'a, ID, ACTUATOR_RATE, Crbm, Cactuator, N_RBM, N_ACTUATOR>
    where
        Crbm: Update + Write<RBM<ID>> + Send + Sync + 'static,
        Cactuator: Update + Write<ActuatorCommandForces<ID>> + Send + Sync + 'static,
    {
        SegmentBuilder {
            rbm_setpoint_actor,
            actuator_setpoint_actor,
            calibration,
        }
    }
}

impl<
        'a,
        const ID: u8,
        const ACTUATOR_RATE: usize,
        Crbm,
        Cactuator,
        const N_RBM: usize,
        const N_ACTUATOR: usize,
    > SegmentBuilder<'a, ID, ACTUATOR_RATE, Crbm, Cactuator, N_RBM, N_ACTUATOR>
where
    Crbm: Update + Write<RBM<ID>> + Send + Sync + 'static,
    Cactuator: Update + Write<ActuatorCommandForces<ID>> + Send + Sync + 'static,
{
    pub fn build(
        self,
        plant: &mut Actor<DiscreteModalSolver<ExponentialMatrix>>,
    ) -> anyhow::Result<SubSystem<SegmentControl<ID, ACTUATOR_RATE>>> {
        let mut sys = SubSystem::new(SegmentControl::<ID, ACTUATOR_RATE>::new(&self.calibration))
            .build()?
            .flowchart();

        self.rbm_setpoint_actor
            .add_output()
            .build::<RBM<ID>>()
            .into_input(&mut sys)?;

        self.actuator_setpoint_actor
            .add_output()
            .build::<ActuatorCommandForces<ID>>()
            .into_input(&mut sys)?;

        sys.add_output()
            .build::<HardpointsForces<ID>>()
            .into_input(plant)?;

        sys.add_output()
            .build::<ActuatorAppliedForces<ID>>()
            .into_input(plant)?;

        plant
            .add_output()
            .bootstrap()
            .build::<HardpointsMotion<ID>>()
            .into_input(&mut sys)?;

        Ok(sys.into())
    }
}
