//! M2  ASM segment

use super::prelude::*;
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    FluidDampingForces, VoiceCoilsForces, VoiceCoilsMotion,
};

impl<const ID: u8, S: Solver + Default> Read<VoiceCoilsForces<ID>> for DiscreteModalSolver<S> {
    fn read(&mut self, data: Arc<Data<VoiceCoilsForces<ID>>>) {
        match ID {
            1 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S1VCDeltaF>>::set(self, &data),
            2 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S2VCDeltaF>>::set(self, &data),
            3 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S3VCDeltaF>>::set(self, &data),
            4 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S4VCDeltaF>>::set(self, &data),
            5 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S5VCDeltaF>>::set(self, &data),
            6 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S6VCDeltaF>>::set(self, &data),
            7 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S7VCDeltaF>>::set(self, &data),
            _ => unreachable!(),
        }
    }
}

impl<const ID: u8, S: Solver + Default> Read<FluidDampingForces<ID>> for DiscreteModalSolver<S> {
    fn read(&mut self, data: Arc<Data<FluidDampingForces<ID>>>) {
        match ID {
            1 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S1FluidDampingF>>::set(self, &data),
            2 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S2FluidDampingF>>::set(self, &data),
            3 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S3FluidDampingF>>::set(self, &data),
            4 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S4FluidDampingF>>::set(self, &data),
            5 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S5FluidDampingF>>::set(self, &data),
            6 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S6FluidDampingF>>::set(self, &data),
            7 => <DiscreteModalSolver<S> as Set<fem_io::MCM2S7FluidDampingF>>::set(self, &data),
            _ => unreachable!(),
        }
    }
}

impl<const ID: u8, S: Solver + Default> Write<VoiceCoilsMotion<ID>> for DiscreteModalSolver<S> {
    fn write(&mut self) -> Option<Arc<Data<VoiceCoilsMotion<ID>>>> {
        match ID {
            1 => <DiscreteModalSolver<S> as Get<fem_io::MCM2S1VCDeltaD>>::get(self),
            2 => <DiscreteModalSolver<S> as Get<fem_io::MCM2S2VCDeltaD>>::get(self),
            3 => <DiscreteModalSolver<S> as Get<fem_io::MCM2S3VCDeltaD>>::get(self),
            4 => <DiscreteModalSolver<S> as Get<fem_io::MCM2S4VCDeltaD>>::get(self),
            5 => <DiscreteModalSolver<S> as Get<fem_io::MCM2S5VCDeltaD>>::get(self),
            6 => <DiscreteModalSolver<S> as Get<fem_io::MCM2S6VCDeltaD>>::get(self),
            7 => <DiscreteModalSolver<S> as Get<fem_io::MCM2S7VCDeltaD>>::get(self),
            _ => unreachable!(),
        }
        .map(|data| Arc::new(Data::new(data)))
    }
}
