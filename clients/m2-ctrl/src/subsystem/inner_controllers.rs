use crate::AsmSegmentInnerController;
use gmt_dos_actors::{framework::model::Check, prelude::*};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    AsmCommand, FluidDampingForces, VoiceCoilsForces, VoiceCoilsMotion,
};

use super::{DispatchIn, DispatchOut};

#[derive(Debug)]
pub enum AsmsInnerControllers {
    S1(Actor<AsmSegmentInnerController<1>>),
    S2(Actor<AsmSegmentInnerController<2>>),
    S3(Actor<AsmSegmentInnerController<3>>),
    S4(Actor<AsmSegmentInnerController<4>>),
    S5(Actor<AsmSegmentInnerController<5>>),
    S6(Actor<AsmSegmentInnerController<6>>),
    S7(Actor<AsmSegmentInnerController<7>>),
}
impl AsmsInnerControllers {
    pub fn new(id: u8, n_mode: usize, ks: Option<Vec<f64>>) -> Self {
        match id {
            1 => Self::S1((AsmSegmentInnerController::<1>::new(n_mode, ks), "ASM #1").into()),
            2 => Self::S2((AsmSegmentInnerController::<2>::new(n_mode, ks), "ASM #2").into()),
            3 => Self::S3((AsmSegmentInnerController::<3>::new(n_mode, ks), "ASM #3").into()),
            4 => Self::S4((AsmSegmentInnerController::<4>::new(n_mode, ks), "ASM #4").into()),
            5 => Self::S5((AsmSegmentInnerController::<5>::new(n_mode, ks), "ASM #5").into()),
            6 => Self::S6((AsmSegmentInnerController::<6>::new(n_mode, ks), "ASM #6").into()),
            7 => Self::S7((AsmSegmentInnerController::<7>::new(n_mode, ks), "ASM #7").into()),
            _ => todo!(),
        }
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
    pub fn asm_command(&mut self, dispatch: &mut Actor<DispatchIn>) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => dispatch
                .add_output()
                .build::<AsmCommand<1>>()
                .into_input(actor)?,
            Self::S2(actor) => dispatch
                .add_output()
                .build::<AsmCommand<2>>()
                .into_input(actor)?,
            Self::S3(actor) => dispatch
                .add_output()
                .build::<AsmCommand<3>>()
                .into_input(actor)?,
            Self::S4(actor) => dispatch
                .add_output()
                .build::<AsmCommand<4>>()
                .into_input(actor)?,
            Self::S5(actor) => dispatch
                .add_output()
                .build::<AsmCommand<5>>()
                .into_input(actor)?,
            Self::S6(actor) => dispatch
                .add_output()
                .build::<AsmCommand<6>>()
                .into_input(actor)?,
            Self::S7(actor) => dispatch
                .add_output()
                .build::<AsmCommand<7>>()
                .into_input(actor)?,
        };
        Ok(())
    }
    pub fn asm_voice_coils_motion(
        &mut self,
        dispatch: &mut Actor<DispatchIn>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => dispatch
                .add_output()
                .build::<VoiceCoilsMotion<1>>()
                .into_input(actor)?,
            Self::S2(actor) => dispatch
                .add_output()
                .build::<VoiceCoilsMotion<2>>()
                .into_input(actor)?,
            Self::S3(actor) => dispatch
                .add_output()
                .build::<VoiceCoilsMotion<3>>()
                .into_input(actor)?,
            Self::S4(actor) => dispatch
                .add_output()
                .build::<VoiceCoilsMotion<4>>()
                .into_input(actor)?,
            Self::S5(actor) => dispatch
                .add_output()
                .build::<VoiceCoilsMotion<5>>()
                .into_input(actor)?,
            Self::S6(actor) => dispatch
                .add_output()
                .build::<VoiceCoilsMotion<6>>()
                .into_input(actor)?,
            Self::S7(actor) => dispatch
                .add_output()
                .build::<VoiceCoilsMotion<7>>()
                .into_input(actor)?,
        };
        Ok(())
    }
    pub fn asm_voice_coils_forces(
        &mut self,
        dispatch: &mut Actor<DispatchOut>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => actor
                .add_output()
                .build::<VoiceCoilsForces<1>>()
                .into_input(dispatch)?,
            Self::S2(actor) => actor
                .add_output()
                .build::<VoiceCoilsForces<2>>()
                .into_input(dispatch)?,
            Self::S3(actor) => actor
                .add_output()
                .build::<VoiceCoilsForces<3>>()
                .into_input(dispatch)?,
            Self::S4(actor) => actor
                .add_output()
                .build::<VoiceCoilsForces<4>>()
                .into_input(dispatch)?,
            Self::S5(actor) => actor
                .add_output()
                .build::<VoiceCoilsForces<5>>()
                .into_input(dispatch)?,
            Self::S6(actor) => actor
                .add_output()
                .build::<VoiceCoilsForces<6>>()
                .into_input(dispatch)?,
            Self::S7(actor) => actor
                .add_output()
                .build::<VoiceCoilsForces<7>>()
                .into_input(dispatch)?,
        };
        Ok(())
    }
    pub fn asm_fluid_damping_forces(
        &mut self,
        dispatch: &mut Actor<DispatchOut>,
    ) -> anyhow::Result<()> {
        match self {
            Self::S1(actor) => actor
                .add_output()
                .build::<FluidDampingForces<1>>()
                .into_input(dispatch)?,
            Self::S2(actor) => actor
                .add_output()
                .build::<FluidDampingForces<2>>()
                .into_input(dispatch)?,
            Self::S3(actor) => actor
                .add_output()
                .build::<FluidDampingForces<3>>()
                .into_input(dispatch)?,
            Self::S4(actor) => actor
                .add_output()
                .build::<FluidDampingForces<4>>()
                .into_input(dispatch)?,
            Self::S5(actor) => actor
                .add_output()
                .build::<FluidDampingForces<5>>()
                .into_input(dispatch)?,
            Self::S6(actor) => actor
                .add_output()
                .build::<FluidDampingForces<6>>()
                .into_input(dispatch)?,
            Self::S7(actor) => actor
                .add_output()
                .build::<FluidDampingForces<7>>()
                .into_input(dispatch)?,
        };
        Ok(())
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
}
