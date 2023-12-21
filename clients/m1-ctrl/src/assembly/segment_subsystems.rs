use gmt_dos_actors::{framework::model::Check, prelude::*};
use gmt_dos_clients_io::gmt_m1::segment::{
    ActuatorAppliedForces, ActuatorCommandForces, HardpointsForces, HardpointsMotion, RBM,
};

use crate::{
    subsystems::{Segment, SegmentSubSystem},
    Calibration,
};

use super::dispatch::{DispatchIn, DispatchOut};

#[derive(Clone)]
pub enum SegmentSubSystems<const R: usize> {
    S1(SegmentSubSystem<1, R>),
    S2(SegmentSubSystem<2, R>),
    S3(SegmentSubSystem<3, R>),
    S4(SegmentSubSystem<4, R>),
    S5(SegmentSubSystem<5, R>),
    S6(SegmentSubSystem<6, R>),
    S7(SegmentSubSystem<7, R>),
}

impl<const R: usize> SegmentSubSystems<R> {
    pub fn new(id: u8, calibration: &Calibration) -> anyhow::Result<Self> {
        Ok(match id {
            1 => Self::S1(Segment::<1, R>::new(calibration)?),
            2 => Self::S2(Segment::<2, R>::new(calibration)?),
            3 => Self::S3(Segment::<3, R>::new(calibration)?),
            4 => Self::S4(Segment::<4, R>::new(calibration)?),
            5 => Self::S5(Segment::<5, R>::new(calibration)?),
            6 => Self::S6(Segment::<6, R>::new(calibration)?),
            7 => Self::S7(Segment::<7, R>::new(calibration)?),
            _ => todo!(),
        })
    }
    pub fn get(&self) -> Option<&dyn Check> {
        match self {
            Self::S1(actor) => Some(actor as &dyn Check),
            Self::S2(actor) => Some(actor as &dyn Check),
            Self::S3(actor) => Some(actor as &dyn Check),
            Self::S4(actor) => Some(actor as &dyn Check),
            Self::S5(actor) => Some(actor as &dyn Check),
            Self::S6(actor) => Some(actor as &dyn Check),
            Self::S7(actor) => Some(actor as &dyn Check),
        }
    }
    pub fn into_model(self) -> Model<Unknown> {
        match self {
            Self::S1(actor) => model!(actor),
            Self::S2(actor) => model!(actor),
            Self::S3(actor) => model!(actor),
            Self::S4(actor) => model!(actor),
            Self::S5(actor) => model!(actor),
            Self::S6(actor) => model!(actor),
            Self::S7(actor) => model!(actor),
        }
    }
    pub fn m1_rigid_body_motions(
        &mut self,
        dispatch: &mut Actor<DispatchIn>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => dispatch.add_output().build::<RBM<1>>().into_input(actor)?,
            Self::S2(actor) => dispatch.add_output().build::<RBM<2>>().into_input(actor)?,
            Self::S3(actor) => dispatch.add_output().build::<RBM<3>>().into_input(actor)?,
            Self::S4(actor) => dispatch.add_output().build::<RBM<4>>().into_input(actor)?,
            Self::S5(actor) => dispatch.add_output().build::<RBM<5>>().into_input(actor)?,
            Self::S6(actor) => dispatch.add_output().build::<RBM<6>>().into_input(actor)?,
            Self::S7(actor) => dispatch.add_output().build::<RBM<7>>().into_input(actor)?,
        };
        Ok(())
    }
    pub fn m1_actuator_command_forces(
        &mut self,
        dispatch: &mut Actor<DispatchIn>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<1>>()
                .into_input(actor)?,
            Self::S2(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<2>>()
                .into_input(actor)?,
            Self::S3(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<3>>()
                .into_input(actor)?,
            Self::S4(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<4>>()
                .into_input(actor)?,
            Self::S5(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<5>>()
                .into_input(actor)?,
            Self::S6(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<6>>()
                .into_input(actor)?,
            Self::S7(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<7>>()
                .into_input(actor)?,
        };
        Ok(())
    }
    pub fn m1_hardpoints_motion(&mut self, dispatch: &mut Actor<DispatchIn>) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<1>>()
                .into_input(actor)?,
            Self::S2(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<2>>()
                .into_input(actor)?,
            Self::S3(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<3>>()
                .into_input(actor)?,
            Self::S4(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<4>>()
                .into_input(actor)?,
            Self::S5(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<5>>()
                .into_input(actor)?,
            Self::S6(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<6>>()
                .into_input(actor)?,
            Self::S7(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<7>>()
                .into_input(actor)?,
        };
        Ok(())
    }
    pub fn m1_actuator_applied_forces(
        &mut self,
        dispatch: &mut Actor<DispatchOut>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => actor
                .add_output()
                .build::<ActuatorAppliedForces<1>>()
                .into_input(dispatch)?,
            Self::S2(actor) => actor
                .add_output()
                .build::<ActuatorAppliedForces<2>>()
                .into_input(dispatch)?,
            Self::S3(actor) => actor
                .add_output()
                .build::<ActuatorAppliedForces<3>>()
                .into_input(dispatch)?,
            Self::S4(actor) => actor
                .add_output()
                .build::<ActuatorAppliedForces<4>>()
                .into_input(dispatch)?,
            Self::S5(actor) => actor
                .add_output()
                .build::<ActuatorAppliedForces<5>>()
                .into_input(dispatch)?,
            Self::S6(actor) => actor
                .add_output()
                .build::<ActuatorAppliedForces<6>>()
                .into_input(dispatch)?,
            Self::S7(actor) => actor
                .add_output()
                .build::<ActuatorAppliedForces<7>>()
                .into_input(dispatch)?,
        };
        Ok(())
    }
    pub fn m1_hardpoints_forces(
        &mut self,
        dispatch: &mut Actor<DispatchOut>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => actor
                .add_output()
                .build::<HardpointsForces<1>>()
                .into_input(dispatch)?,
            Self::S2(actor) => actor
                .add_output()
                .build::<HardpointsForces<2>>()
                .into_input(dispatch)?,
            Self::S3(actor) => actor
                .add_output()
                .build::<HardpointsForces<3>>()
                .into_input(dispatch)?,
            Self::S4(actor) => actor
                .add_output()
                .build::<HardpointsForces<4>>()
                .into_input(dispatch)?,
            Self::S5(actor) => actor
                .add_output()
                .build::<HardpointsForces<5>>()
                .into_input(dispatch)?,
            Self::S6(actor) => actor
                .add_output()
                .build::<HardpointsForces<6>>()
                .into_input(dispatch)?,
            Self::S7(actor) => actor
                .add_output()
                .build::<HardpointsForces<7>>()
                .into_input(dispatch)?,
        };
        Ok(())
    }
}
