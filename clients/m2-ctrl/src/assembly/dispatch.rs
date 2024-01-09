use std::sync::Arc;

use gmt_dos_clients_io::{
    gmt_m2::asm::{
        segment::{AsmCommand, FluidDampingForces, VoiceCoilsForces, VoiceCoilsMotion},
        M2ASMAsmCommand, M2ASMFluidDampingForces, M2ASMVoiceCoilsForces, M2ASMVoiceCoilsMotion,
    },
    Assembly,
};
use interface::{Data, Read, Update, Write};

impl Assembly for DispatchIn {}
impl Assembly for DispatchOut {}

#[derive(Debug, Default)]
pub struct DispatchIn
where
    Self: Assembly,
{
    asms_command: Arc<Vec<Arc<Vec<f64>>>>,
    asms_voice_coil_motion: Arc<Vec<Arc<Vec<f64>>>>,
}

#[derive(Debug, Default)]
pub struct DispatchOut
where
    Self: Assembly,
{
    asms_voice_coil_forces: Vec<Arc<Vec<f64>>>,
    asms_fluid_damping_forces: Vec<Arc<Vec<f64>>>,
}

impl DispatchIn {
    pub fn new() -> Self {
        Default::default()
    }
}

impl DispatchOut {
    const NA: usize = 675;

    pub fn new() -> Self {
        Self {
            asms_voice_coil_forces: vec![
                Arc::new(Vec::with_capacity(Self::NA));
                <Self as Assembly>::N
            ],
            asms_fluid_damping_forces: vec![
                Arc::new(Vec::with_capacity(Self::NA));
                <Self as Assembly>::N
            ],
        }
    }
}

impl Update for DispatchIn {}
impl Update for DispatchOut {}

impl Read<M2ASMVoiceCoilsMotion> for DispatchIn {
    fn read(&mut self, data: Data<M2ASMVoiceCoilsMotion>) {
        self.asms_voice_coil_motion = data.into_arc();
    }
}
impl<const ID: u8> Write<VoiceCoilsMotion<ID>> for DispatchIn {
    fn write(&mut self) -> Option<Data<VoiceCoilsMotion<ID>>> {
        <Self as Assembly>::position::<ID>().and_then(|idx| {
            self.asms_voice_coil_motion
                .get(idx)
                .map(|data| data.clone().into())
        })
    }
}

impl Read<M2ASMAsmCommand> for DispatchIn {
    fn read(&mut self, data: Data<M2ASMAsmCommand>) {
        self.asms_command = data.into_arc();
    }
}
impl<const ID: u8> Write<AsmCommand<ID>> for DispatchIn {
    fn write(&mut self) -> Option<Data<AsmCommand<ID>>> {
        <Self as Assembly>::position::<ID>()
            .and_then(|idx| self.asms_command.get(idx).map(|data| data.clone().into()))
    }
}

impl<const ID: u8> Read<VoiceCoilsForces<ID>> for DispatchOut {
    fn read(&mut self, data: Data<VoiceCoilsForces<ID>>) {
        if let Some(idx) = <Self as Assembly>::position::<ID>() {
            let forces = data.into_arc();
            self.asms_voice_coil_forces[idx] = forces;
        }
    }
}
impl Write<M2ASMVoiceCoilsForces> for DispatchOut {
    fn write(&mut self) -> Option<Data<M2ASMVoiceCoilsForces>> {
        Some(Data::new(self.asms_voice_coil_forces.clone()))
    }
}

impl<const ID: u8> Read<FluidDampingForces<ID>> for DispatchOut {
    fn read(&mut self, data: Data<FluidDampingForces<ID>>) {
        if let Some(idx) = <Self as Assembly>::position::<ID>() {
            let forces = data.into_arc();
            self.asms_fluid_damping_forces[idx] = forces;
        }
    }
}
impl Write<M2ASMFluidDampingForces> for DispatchOut {
    fn write(&mut self) -> Option<Data<M2ASMFluidDampingForces>> {
        Some(Data::new(self.asms_fluid_damping_forces.clone()))
    }
}
