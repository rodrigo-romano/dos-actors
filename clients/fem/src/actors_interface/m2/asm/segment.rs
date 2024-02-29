//! M2  ASM segment

use super::prelude::*;
use geotrans::{Quaternion, Vector};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    FaceSheetFigure, FluidDampingForces, VoiceCoilsForces, VoiceCoilsMotion,
};

impl<const ID: u8, S> Read<VoiceCoilsForces<ID>> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn read(&mut self, data: Data<VoiceCoilsForces<ID>>) {
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

impl<const ID: u8, S> Size<VoiceCoilsForces<ID>> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn len(&self) -> usize {
        675
    }
}

impl<const ID: u8, S> Read<FluidDampingForces<ID>> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn read(&mut self, data: Data<FluidDampingForces<ID>>) {
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

impl<const ID: u8, S> Size<FluidDampingForces<ID>> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn len(&self) -> usize {
        675
    }
}
impl<const ID: u8, S> Write<VoiceCoilsMotion<ID>> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn write(&mut self) -> Option<Data<VoiceCoilsMotion<ID>>> {
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
        .map(|data| Data::new(data))
    }
}

impl<const ID: u8, S> Size<VoiceCoilsMotion<ID>> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn len(&self) -> usize {
        675
    }
}

/* impl<const ID: u8, S> Write<FaceSheetFigure<ID>> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn write(&mut self) -> Option<Data<FaceSheetFigure<ID>>> {
        match ID {
            1 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment1AxialD>>::get(self),
            2 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment2AxialD>>::get(self),
            3 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment3AxialD>>::get(self),
            4 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment4AxialD>>::get(self),
            5 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment5AxialD>>::get(self),
            6 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment6AxialD>>::get(self),
            7 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment7AxialD>>::get(self),
            _ => unreachable!(),
        }
        .map(|data| Data::new(data))
    }
} */

impl<const ID: u8, S> Write<FaceSheetFigure<ID>> for DiscreteModalSolver<S>
where
    S: Solver + Default,
    DiscreteModalSolver<S>: Iterator,
{
    fn write(&mut self) -> Option<Data<FaceSheetFigure<ID>>> {
        fn rbm_removal(rbm: &[f64], nodes: &mut [f64], figure: &[f64]) -> Vec<f64> {
            let tz = rbm[2];
            let q = Quaternion::unit(rbm[5], Vector::k())
                * Quaternion::unit(rbm[4], Vector::j())
                * Quaternion::unit(rbm[3], Vector::i());
            nodes
                .chunks_mut(3)
                .zip(figure)
                .map(|(u, dz)| {
                    u[2] = dz - tz;
                    let p: Quaternion = From::<&[f64]>::from(u);
                    let pp = q.complex_conjugate() * p * &q;
                    let v: Vec<f64> = pp.vector_as_slice().to_vec();
                    v[2]
                })
                .collect()
        }

        let figure = match ID {
            1 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment1AxialD>>::get(self),
            2 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment2AxialD>>::get(self),
            3 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment3AxialD>>::get(self),
            4 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment4AxialD>>::get(self),
            5 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment5AxialD>>::get(self),
            6 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment6AxialD>>::get(self),
            7 => <DiscreteModalSolver<S> as Get<fem_io::M2Segment7AxialD>>::get(self),
            _ => unreachable!(),
        }?;

        if self.facesheet_nodes.is_some() {
            let rbms = <DiscreteModalSolver<S> as Get<fem_io::MCM2Lcl6D>>::get(self)
                .expect("failed to get rigid body motion from ASMS reference bodies");
            let rbm = rbms
                .chunks(6)
                .nth(ID as usize - 1)
                .expect("failed to get rigid body motion from ASM reference body #{ID");
            let nodes = self
                .facesheet_nodes
                .as_mut()
                .expect("facesheet nodes are missing")
                .get_mut(&ID)?;
            Some(rbm_removal(&rbm, nodes, &figure).into())
        } else {
            Some(figure.into())
        }
    }
}
