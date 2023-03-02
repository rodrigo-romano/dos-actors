use crate::{Actuators, Hardpoints};
use crate::{Calibration, LoadCells, Segment};
use gmt_dos_actors::model;
use gmt_dos_actors::prelude::Model;
use gmt_dos_actors::{model::Unknown, Actor, AddOuput, TryIntoInputs};
use gmt_dos_clients::interface::{Update, Write};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m1::segment::{
    ActuatorAppliedForces, ActuatorCommandForces, BarycentricForce, HardpointsForces,
    HardpointsMotion, RBM,
};

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
    Crbm: Update + Write<RBM<ID>> + Send + 'static,
    Cactuator: Update + Write<ActuatorCommandForces<ID>> + Send + 'static,
{
    rbm_setpoint_actor: &'a mut Actor<Crbm, N_RBM, 1>,
    actuator_setpoint_actor: &'a mut Actor<Cactuator, N_ACTUATOR, ACTUATOR_RATE>,
    calibration: Calibration,
}

impl<'a, const ID: u8, const ACTUATOR_RATE: usize> Segment<ID, ACTUATOR_RATE> {
    pub fn builder<Crbm, Cactuator, const N_ACTUATOR: usize, const N_RBM: usize>(
        calibration: Calibration,
        rbm_setpoint_actor: &'a mut Actor<Crbm, N_RBM, 1>,
        actuator_setpoint_actor: &'a mut Actor<Cactuator, N_ACTUATOR, ACTUATOR_RATE>,
    ) -> SegmentBuilder<'a, ID, ACTUATOR_RATE, Crbm, Cactuator, N_RBM, N_ACTUATOR>
    where
        Crbm: Update + Write<RBM<ID>> + Send + 'static,
        Cactuator: Update + Write<ActuatorCommandForces<ID>> + Send + 'static,
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
    Crbm: Update + Write<RBM<ID>> + Send + 'static,
    Cactuator: Update + Write<ActuatorCommandForces<ID>> + Send + 'static,
{
    pub fn build(
        self,
        plant: &mut Actor<DiscreteModalSolver<ExponentialMatrix>>,
    ) -> anyhow::Result<Model<Unknown>> {
        let Calibration {
            stiffness,
            rbm_2_hp,
            lc_2_cg,
        } = self.calibration;
        let mut hardpoints: Actor<_> = (
            Hardpoints::new(stiffness, rbm_2_hp[ID as usize - 1]),
            format!(
                "M1S{ID}
                    Hardpoints"
            ),
        )
            .into();

        let mut loadcells: Actor<_, 1, ACTUATOR_RATE> = (
            LoadCells::new(stiffness, lc_2_cg[ID as usize - 1]),
            format!(
                "M1S{ID}
                    Loadcells"
            ),
        )
            .into();

        let mut actuators: Actor<_, ACTUATOR_RATE, 1> = (
            Actuators::<ID>::new(),
            format!(
                "M1S{ID}
                    Actuators"
            ),
        )
            .into();

        self.rbm_setpoint_actor
            .add_output()
            .build::<RBM<ID>>()
            .into_input(&mut hardpoints)?;

        self.actuator_setpoint_actor
            .add_output()
            .build::<ActuatorCommandForces<ID>>()
            .into_input(&mut actuators)?;

        hardpoints
            .add_output()
            .multiplex(2)
            .build::<HardpointsForces<ID>>()
            .into_input(&mut loadcells)
            .into_input(plant)?;

        loadcells
            .add_output()
            .bootstrap()
            .build::<BarycentricForce<ID>>()
            .into_input(&mut actuators)?;

        actuators
            .add_output()
            .build::<ActuatorAppliedForces<ID>>()
            .into_input(plant)?;

        plant
            .add_output()
            .bootstrap()
            .build::<HardpointsMotion<ID>>()
            .into_input(&mut loadcells)?;

        Ok(model!(hardpoints, loadcells, actuators))
    }
}
