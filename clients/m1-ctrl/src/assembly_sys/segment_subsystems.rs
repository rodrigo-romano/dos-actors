use gmt_dos_actors::{framework::model::Check, prelude::*, system::Sys, Task};
use gmt_dos_clients::Sampler;
use gmt_dos_clients_io::gmt_m1::segment::{
    ActuatorAppliedForces, ActuatorCommandForces, HardpointsForces, HardpointsMotion, RBM,
};

use crate::{subsystems::SegmentControl, Actuators, Calibration, Hardpoints, LoadCells};

use super::dispatch::{DispatchIn, DispatchOut};

#[derive(Clone)]
pub enum SegmentControls<const R: usize> {
    S1(Sys<SegmentControl<1, R>>),
    S2(Sys<SegmentControl<2, R>>),
    S3(Sys<SegmentControl<3, R>>),
    S4(Sys<SegmentControl<4, R>>),
    S5(Sys<SegmentControl<5, R>>),
    S6(Sys<SegmentControl<6, R>>),
    S7(Sys<SegmentControl<7, R>>),
}

impl<'a, const R: usize> IntoIterator for &'a SegmentControls<R> {
    type Item = Box<&'a dyn Check>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match &self {
            SegmentControls::S1(segment) => segment.into_iter(),
            SegmentControls::S2(segment) => segment.into_iter(),
            SegmentControls::S3(segment) => segment.into_iter(),
            SegmentControls::S4(segment) => segment.into_iter(),
            SegmentControls::S5(segment) => segment.into_iter(),
            SegmentControls::S6(segment) => segment.into_iter(),
            SegmentControls::S7(segment) => segment.into_iter(),
        }
    }
}

impl<const R: usize> IntoIterator for Box<SegmentControls<R>> {
    type Item = Box<dyn Task>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match *self {
            SegmentControls::S1(segment) => Box::new(segment).into_iter(),
            SegmentControls::S2(segment) => Box::new(segment).into_iter(),
            SegmentControls::S3(segment) => Box::new(segment).into_iter(),
            SegmentControls::S4(segment) => Box::new(segment).into_iter(),
            SegmentControls::S5(segment) => Box::new(segment).into_iter(),
            SegmentControls::S6(segment) => Box::new(segment).into_iter(),
            SegmentControls::S7(segment) => Box::new(segment).into_iter(),
        }
    }
}

impl<const R: usize> SegmentControls<R> {
    pub fn new(id: u8, calibration: &Calibration) -> anyhow::Result<Self> {
        Ok(match id {
            1 => Self::S1(
                Sys::new(SegmentControl::<1, R>::new(calibration))
                    .build()?
                    .flowchart(),
            ),
            2 => Self::S2(Sys::new(SegmentControl::<2, R>::new(calibration)).build()?),
            3 => Self::S3(Sys::new(SegmentControl::<3, R>::new(calibration)).build()?),
            4 => Self::S4(Sys::new(SegmentControl::<4, R>::new(calibration)).build()?),
            5 => Self::S5(Sys::new(SegmentControl::<5, R>::new(calibration)).build()?),
            6 => Self::S6(Sys::new(SegmentControl::<6, R>::new(calibration)).build()?),
            7 => Self::S7(Sys::new(SegmentControl::<7, R>::new(calibration)).build()?),
            _ => todo!(),
        })
    }
    /*     pub fn get(&self) -> Option<&dyn Check> {
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
    } */
    pub fn m1_rigid_body_motions(
        &mut self,
        dispatch: &mut Actor<DispatchIn>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => dispatch
                .add_output()
                .build::<RBM<1>>()
                .into_input::<Hardpoints>(actor)?,
            Self::S2(actor) => dispatch
                .add_output()
                .build::<RBM<2>>()
                .into_input::<Hardpoints>(actor)?,
            Self::S3(actor) => dispatch
                .add_output()
                .build::<RBM<3>>()
                .into_input::<Hardpoints>(actor)?,
            Self::S4(actor) => dispatch
                .add_output()
                .build::<RBM<4>>()
                .into_input::<Hardpoints>(actor)?,
            Self::S5(actor) => dispatch
                .add_output()
                .build::<RBM<5>>()
                .into_input::<Hardpoints>(actor)?,
            Self::S6(actor) => dispatch
                .add_output()
                .build::<RBM<6>>()
                .into_input::<Hardpoints>(actor)?,
            Self::S7(actor) => dispatch
                .add_output()
                .build::<RBM<7>>()
                .into_input::<Hardpoints>(actor)?,
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
                .into_input::<Sampler<Vec<f64>, ActuatorCommandForces<1>>>(actor)?,
            Self::S2(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<2>>()
                .into_input::<Sampler<Vec<f64>, ActuatorCommandForces<2>>>(actor)?,
            Self::S3(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<3>>()
                .into_input::<Sampler<Vec<f64>, ActuatorCommandForces<3>>>(actor)?,
            Self::S4(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<4>>()
                .into_input::<Sampler<Vec<f64>, ActuatorCommandForces<4>>>(actor)?,
            Self::S5(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<5>>()
                .into_input::<Sampler<Vec<f64>, ActuatorCommandForces<5>>>(actor)?,
            Self::S6(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<6>>()
                .into_input::<Sampler<Vec<f64>, ActuatorCommandForces<6>>>(actor)?,
            Self::S7(actor) => dispatch
                .add_output()
                .build::<ActuatorCommandForces<7>>()
                .into_input::<Sampler<Vec<f64>, ActuatorCommandForces<7>>>(actor)?,
        }; /*         match self {
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
           }; */
        Ok(())
    }
    pub fn m1_hardpoints_motion(&mut self, dispatch: &mut Actor<DispatchIn>) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<1>>()
                .into_input::<LoadCells>(actor)?,
            Self::S2(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<2>>()
                .into_input::<LoadCells>(actor)?,
            Self::S3(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<3>>()
                .into_input::<LoadCells>(actor)?,
            Self::S4(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<4>>()
                .into_input::<LoadCells>(actor)?,
            Self::S5(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<5>>()
                .into_input::<LoadCells>(actor)?,
            Self::S6(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<6>>()
                .into_input::<LoadCells>(actor)?,
            Self::S7(actor) => dispatch
                .add_output()
                .build::<HardpointsMotion<7>>()
                .into_input::<LoadCells>(actor)?,
        }; /*         match self {
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
           }; */
        Ok(())
    }
    pub fn m1_actuator_applied_forces(
        &mut self,
        dispatch: &mut Actor<DispatchOut>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => AddActorOutput::<'_, Actuators<1>, R, 1>::add_output(actor)
                .build::<ActuatorAppliedForces<1>>()
                .into_input(dispatch)?,
            Self::S2(actor) => AddActorOutput::<'_, Actuators<2>, R, 1>::add_output(actor)
                .build::<ActuatorAppliedForces<2>>()
                .into_input(dispatch)?,
            Self::S3(actor) => AddActorOutput::<'_, Actuators<3>, R, 1>::add_output(actor)
                .build::<ActuatorAppliedForces<3>>()
                .into_input(dispatch)?,
            Self::S4(actor) => AddActorOutput::<'_, Actuators<4>, R, 1>::add_output(actor)
                .build::<ActuatorAppliedForces<4>>()
                .into_input(dispatch)?,
            Self::S5(actor) => AddActorOutput::<'_, Actuators<5>, R, 1>::add_output(actor)
                .build::<ActuatorAppliedForces<5>>()
                .into_input(dispatch)?,
            Self::S6(actor) => AddActorOutput::<'_, Actuators<6>, R, 1>::add_output(actor)
                .build::<ActuatorAppliedForces<6>>()
                .into_input(dispatch)?,
            Self::S7(actor) => AddActorOutput::<'_, Actuators<7>, R, 1>::add_output(actor)
                .build::<ActuatorAppliedForces<7>>()
                .into_input(dispatch)?,
        };
        /*         match self {
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
        }; */
        Ok(())
    }
    pub fn m1_hardpoints_forces(
        &mut self,
        dispatch: &mut Actor<DispatchOut>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => AddActorOutput::<'_, Hardpoints, 1, 1>::add_output(actor)
                .build::<HardpointsForces<1>>()
                .into_input(dispatch)?,
            Self::S2(actor) => AddActorOutput::<'_, Hardpoints, 1, 1>::add_output(actor)
                .build::<HardpointsForces<2>>()
                .into_input(dispatch)?,
            Self::S3(actor) => AddActorOutput::<'_, Hardpoints, 1, 1>::add_output(actor)
                .build::<HardpointsForces<3>>()
                .into_input(dispatch)?,
            Self::S4(actor) => AddActorOutput::<'_, Hardpoints, 1, 1>::add_output(actor)
                .build::<HardpointsForces<4>>()
                .into_input(dispatch)?,
            Self::S5(actor) => AddActorOutput::<'_, Hardpoints, 1, 1>::add_output(actor)
                .build::<HardpointsForces<5>>()
                .into_input(dispatch)?,
            Self::S6(actor) => AddActorOutput::<'_, Hardpoints, 1, 1>::add_output(actor)
                .build::<HardpointsForces<6>>()
                .into_input(dispatch)?,
            Self::S7(actor) => AddActorOutput::<'_, Hardpoints, 1, 1>::add_output(actor)
                .build::<HardpointsForces<7>>()
                .into_input(dispatch)?,
        }; /*         match self {
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
           }; */
        Ok(())
    }
}
