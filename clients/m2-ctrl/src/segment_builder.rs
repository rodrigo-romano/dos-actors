use gmt_dos_actors::{Actor, AddOuput, TryIntoInputs};
use gmt_dos_clients::interface::{Update, Write};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    FluidDampingForces, ModalCommand, VoiceCoilsForces, VoiceCoilsMotion,
};

use crate::{AsmSegmentInnerController, Segment, SegmentCalibration};

pub struct SegmentBuilder<'a, const ID: u8, C, const N: usize>
where
    C: Update + Write<ModalCommand<ID>> + Send + 'static,
{
    calibration: SegmentCalibration,
    n_mode: usize,
    setpoint_actor: &'a mut Actor<C, N, 1>,
}

impl<'a, const ID: u8, C, const N: usize> SegmentBuilder<'a, ID, C, N>
where
    C: Update + Write<ModalCommand<ID>> + Send + 'static,
{
    /// Returns a mount actor
    ///
    ///  The `MountEncoders` input and `MountTorques` output of the mount actor are linked to the plant
    pub fn build(
        self,
        plant: &mut Actor<DiscreteModalSolver<ExponentialMatrix>>,
    ) -> anyhow::Result<Actor<AsmSegmentInnerController<ID>>> {
        let mut asm: Actor<_> = (
            AsmSegmentInnerController::<ID>::new(self.n_mode, Some(self.calibration.stiffness)),
            format!(
                "ASM
     Segment #{ID}"
            ),
        )
            .into();
        self.setpoint_actor
            .add_output()
            .build::<ModalCommand<ID>>()
            .into_input(&mut asm)?;
        asm.add_output()
            .build::<VoiceCoilsForces<ID>>()
            .into_input(plant)?;
        asm.add_output()
            .build::<FluidDampingForces<ID>>()
            .into_input(plant)?;
        plant
            .add_output()
            .bootstrap()
            .build::<VoiceCoilsMotion<ID>>()
            .into_input(&mut asm)?;
        Ok(asm)
    }
}

impl<'a, const ID: u8> Segment<ID> {
    pub fn builder<C, const N: usize>(
        n_mode: usize,
        calibration: SegmentCalibration,
        setpoint_actor: &'a mut Actor<C, N, 1>,
    ) -> SegmentBuilder<'a, ID, C, N>
    where
        C: Update + Write<ModalCommand<ID>> + Send + 'static,
    {
        SegmentBuilder {
            calibration,
            n_mode,
            setpoint_actor,
        }
    }
}
