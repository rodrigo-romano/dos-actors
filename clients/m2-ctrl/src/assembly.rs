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

impl<const R: usize> Assembly for ASMS<R> {}

#[derive(Debug, Clone)]
pub struct ASMS<const R: usize = 1>
where
    Self: Assembly,
{
    segments: Vec<AsmsInnerControllers<R>>,
    dispatch_in: Actor<DispatchIn, R, R>,
    dispatch_out: Actor<DispatchOut, R, R>,
}

impl<const R: usize> ASMS<R> {
    pub fn new(n_mode: Vec<usize>, ks: Vec<Option<Vec<f64>>>) -> Self {
        Self {
            segments: n_mode
                .into_iter()
                .zip(ks.into_iter())
                .zip(<ASMS<R> as Assembly>::SIDS.into_iter())
                .map(|((n_mode, ks), sid)| AsmsInnerControllers::new(sid, n_mode, ks))
                .collect::<Vec<_>>(),
            dispatch_in: DispatchIn::new().into(),
            dispatch_out: DispatchOut::new().into(),
        }
    }
}

impl<const R: usize> GetField for ASMS<R> {
    fn get_field(&self, idx: usize) -> Option<&dyn Check> {
        match idx {
            n if n < <ASMS<R> as Assembly>::N => {
                self.segments.get(idx).and_then(|segment| segment.get())
            }
            n if n == <ASMS<R> as Assembly>::N => Some(&self.dispatch_in as &dyn Check),
            n if n == <ASMS<R> as Assembly>::N + 1 => Some(&self.dispatch_out as &dyn Check),
            _ => None,
        }
    }
}

impl<const R: usize> From<ASMS<R>> for Model<Unknown> {
    fn from(asms: ASMS<R>) -> Self {
        asms.segments
            .into_iter()
            .fold(Model::default(), |model, segment| {
                model + segment.into_model()
            })
            + asms.dispatch_in
            + asms.dispatch_out
    }
}

impl<const R: usize> gateway::Gateways for ASMS<R> {
    type DataType = Vec<Arc<Vec<f64>>>;

    const N_IN: usize = 2;

    const N_OUT: usize = 2;
}

impl<const R: usize> BuildSystem<ASMS<R>, R, R> for ASMS<R> {
    fn build(
        &mut self,
        gateway_in: &mut Actor<gateway::WayIn<ASMS<R>>, R, R>,
        gateway_out: &mut Actor<gateway::WayOut<ASMS<R>>, R, R>,
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
