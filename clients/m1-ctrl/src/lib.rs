//! # M1 control system

mod actuators;
pub use actuators::Actuators;
pub use hardpoints::{Hardpoints, LoadCells};

mod calibration;
mod hardpoints;
mod segment_builder;
pub use calibration::Calibration;
// mod builder;
// use builder::Builder;

pub struct Segment<const ID: u8, const ACTUATOR_RATE: usize>;
impl<const ID: u8, const ACTUATOR_RATE: usize> Segment<ID, ACTUATOR_RATE> {
    pub fn new() -> Self {
        Self
    }
}

pub struct Mirror<const ACTUATOR_RATE: usize> {}

/* impl<'a, const ACTUATOR_RATE: usize> Mirror<ACTUATOR_RATE> {
    pub fn builder<Crbm, Cactuator, const N_ACTUATOR: usize, const N_RBM: usize>(
        fem: &mut FEM,
        rbm_setpoint_actor: &'a mut Actor<Crbm, N_RBM, 1>,
        actuator_setpoint_actor: &'a mut Actor<Cactuator, N_ACTUATOR, ACTUATOR_RATE>,
    ) -> SegmentBuilder<'a, ID, ACTUATOR_RATE, Crbm, Cactuator, N_RBM, N_ACTUATOR>
    where
        Crbm: Update + Write<RBM<ID>> + Send + 'static,
        Cactuator: Update + Write<ActuatorCommandForces<ID>> + Send + 'static,
    {
        let calibration = Calibration::new(fem);
        SegmentBuilder {
            rbm_setpoint_actor,
            actuator_setpoint_actor,
            calibration: Calibration::new(fem),
        }
    }
}
 */
