use std::sync::Arc;

use gmt_dos_actors::{
    actor::Actor,
    framework::{
        model::Check,
        network::{AddActorOutput, AddOuput},
    },
    model::{Model, Unknown},
    prelude::*,
    subsystem::{gateway, BuildSystem, GetField},
};
use gmt_dos_clients_io::{
    gmt_m2::asm::{
        M2ASMAsmCommand, M2ASMFluidDampingForces, M2ASMVoiceCoilsForces, M2ASMVoiceCoilsMotion,
    },
    Assembly,
};

mod dispatch;
mod inner_controllers;
pub use dispatch::{DispatchIn, DispatchOut};
pub use inner_controllers::AsmsInnerControllers;

impl Assembly for ASMS {}

#[derive(Debug)]
pub struct ASMS
where
    Self: Assembly,
{
    segments: [AsmsInnerControllers; <ASMS as Assembly>::N],
    dispatch_in: Actor<DispatchIn>,
    dispatch_out: Actor<DispatchOut>,
}

impl ASMS {
    pub fn new(n_mode: Vec<usize>, ks: Vec<Option<Vec<f64>>>) -> Self {
        Self {
            segments: n_mode
                .into_iter()
                .zip(ks.into_iter())
                .zip(<ASMS as Assembly>::SIDS.into_iter())
                .map(|((n_mode, ks), sid)| AsmsInnerControllers::new(sid, n_mode, ks))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            dispatch_in: DispatchIn::new().into(),
            dispatch_out: DispatchOut::new().into(),
        }
    }
}

impl GetField for ASMS {
    fn get_field(&self, idx: usize) -> Option<&dyn Check> {
        match idx {
            n if n < <ASMS as Assembly>::N => {
                self.segments.get(idx).and_then(|segment| segment.get())
            }
            n if n == <ASMS as Assembly>::N => Some(&self.dispatch_in as &dyn Check),
            n if n == <ASMS as Assembly>::N + 1 => Some(&self.dispatch_out as &dyn Check),
            _ => None,
        }
    }
}

impl From<ASMS> for Model<Unknown> {
    fn from(asms: ASMS) -> Self {
        asms.segments
            .into_iter()
            .fold(Model::default(), |model, segment| {
                model + segment.into_model()
            })
            + asms.dispatch_in
            + asms.dispatch_out
    }
}

impl gateway::Gateways for ASMS {
    type DataType = Vec<Arc<Vec<f64>>>;

    const N_IN: usize = 2;

    const N_OUT: usize = 2;
}

impl BuildSystem<ASMS> for ASMS {
    fn build(
        &mut self,
        gateway_in: &mut Actor<gateway::WayIn<ASMS>, 1, 1>,
        gateway_out: &mut Actor<gateway::WayOut<ASMS>, 1, 1>,
    ) -> anyhow::Result<()> {
        gateway_in
            .add_output()
            .build::<M2ASMAsmCommand>()
            .into_input(&mut self.dispatch_in)?;
        gateway_in
            .add_output()
            .build::<M2ASMVoiceCoilsMotion>()
            .into_input(&mut self.dispatch_in)?;
        self.segments
            .iter_mut()
            .map(|segment| segment.asm_command(&mut self.dispatch_in))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.asm_voice_coils_motion(&mut self.dispatch_in))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.asm_voice_coils_forces(&mut self.dispatch_out))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.asm_fluid_damping_forces(&mut self.dispatch_out))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.dispatch_out
            .add_output()
            .build::<M2ASMVoiceCoilsForces>()
            .into_input(gateway_out)?;
        self.dispatch_out
            .add_output()
            .build::<M2ASMFluidDampingForces>()
            .into_input(gateway_out)?;
        Ok(())
    }
}
