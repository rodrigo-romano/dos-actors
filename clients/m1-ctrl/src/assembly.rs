use std::sync::Arc;

use gmt_dos_actors::{
    framework::model::Check,
    prelude::*,
    subsystem::{gateway, BuildSystem, GetField},
};

use gmt_dos_clients_io::{
    gmt_m1::assembly::{
        M1ActuatorAppliedForces, M1ActuatorCommandForces, M1HardpointsForces, M1HardpointsMotion,
        M1RigidBodyMotions,
    },
    Assembly,
};

mod dispatch;
mod segment_subsystems;
use dispatch::{DispatchIn, DispatchOut};
use segment_subsystems::SegmentSubSystems;

use crate::Calibration;

impl<const R: usize> Assembly for M1<R> {}

pub struct M1<const R: usize>
where
    Self: Assembly,
{
    segments: Vec<SegmentSubSystems<R>>,
    dispatch_in: Actor<DispatchIn>,
    dispatch_out: Actor<DispatchOut>,
}

impl<const R: usize> M1<R> {
    pub fn new(calibration: &Calibration) -> anyhow::Result<Self> {
        Ok(Self {
            segments: <M1<R> as Assembly>::SIDS
                .into_iter()
                .map(|sid| SegmentSubSystems::new(sid, calibration))
                .collect::<anyhow::Result<Vec<_>>>()?,
            dispatch_in: DispatchIn::new().into(),
            dispatch_out: DispatchOut::new().into(),
        })
    }
}

impl<const R: usize> GetField for M1<R> {
    fn get_field(&self, idx: usize) -> Option<&dyn Check> {
        match idx {
            n if n < <M1<R> as Assembly>::N => {
                self.segments.get(idx).and_then(|segment| segment.get())
            }
            n if n == <M1<R> as Assembly>::N => Some(&self.dispatch_in as &dyn Check),
            n if n == <M1<R> as Assembly>::N + 1 => Some(&self.dispatch_out as &dyn Check),
            _ => None,
        }
    }
}

impl<const R: usize> From<M1<R>> for Model<Unknown> {
    fn from(m1: M1<R>) -> Self {
        m1.segments
            .into_iter()
            .fold(Model::default(), |model, segment| {
                model + segment.into_model()
            })
            + m1.dispatch_in
            + m1.dispatch_out
    }
}

impl<const R: usize> gateway::Gateways for M1<R> {
    type DataType = Vec<Arc<Vec<f64>>>;

    const N_IN: usize = 3;

    const N_OUT: usize = 2;
}

impl<const R: usize> BuildSystem<M1<R>> for M1<R> {
    fn build(
        &mut self,
        gateway_in: &mut Actor<gateway::WayIn<M1<R>>, 1, 1>,
        gateway_out: &mut Actor<gateway::WayOut<M1<R>>, 1, 1>,
    ) -> anyhow::Result<()> {
        gateway_in
            .add_output()
            .build::<M1RigidBodyMotions>()
            .into_input(&mut self.dispatch_in)?;
        gateway_in
            .add_output()
            .build::<M1ActuatorCommandForces>()
            .into_input(&mut self.dispatch_in)?;
        gateway_in
            .add_output()
            .build::<M1HardpointsMotion>()
            .into_input(&mut self.dispatch_in)?;

        self.segments
            .iter_mut()
            .map(|segment| segment.m1_rigid_body_motions(&mut self.dispatch_in))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.m1_actuator_command_forces(&mut self.dispatch_in))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.m1_hardpoints_motion(&mut self.dispatch_in))
            .collect::<anyhow::Result<Vec<()>>>()?;

        self.segments
            .iter_mut()
            .map(|segment| segment.m1_actuator_applied_forces(&mut self.dispatch_out))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.m1_hardpoints_forces(&mut self.dispatch_out))
            .collect::<anyhow::Result<Vec<()>>>()?;

        self.dispatch_out
            .add_output()
            .build::<M1ActuatorAppliedForces>()
            .into_input(gateway_out)?;
        self.dispatch_out
            .add_output()
            .build::<M1HardpointsForces>()
            .into_input(gateway_out)?;

        Ok(())
    }
}
