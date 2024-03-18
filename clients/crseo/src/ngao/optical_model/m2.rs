use crseo::SegmentWiseSensor;
use gmt_dos_clients_io::{
    gmt_m2::{
        asm::{segment::FaceSheetFigure, M2ASMFaceSheetFigure, M2ASMReferenceBodyNodes},
        M2RigidBodyMotions,
    },
    optics::M2modes,
};
use interface::{Data, Read};

use crate::OpticalModel;

impl<T: SegmentWiseSensor> Read<M2modes> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2modes>) {
        if 7 * self.gmt.m2_n_mode > data.len() {
            let augmented_data: Vec<_> = data
                .chunks(data.len() / 7)
                .flat_map(|data| {
                    let mut v = vec![0f64];
                    v.extend_from_slice(data);
                    v
                })
                .collect();
            assert_eq!(augmented_data.len(), self.gmt.m2_n_mode * 7);
            self.gmt.m2_modes(&augmented_data);
        } else {
            self.gmt.m2_modes(&data);
        }
    }
}
impl<T: SegmentWiseSensor, const ID: u8> Read<FaceSheetFigure<ID>> for OpticalModel<T> {
    fn read(&mut self, data: Data<FaceSheetFigure<ID>>) {
        self.gmt.m2_segment_modes(ID, &data);
    }
}
impl<T: SegmentWiseSensor> Read<M2ASMFaceSheetFigure> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2ASMFaceSheetFigure>) {
        let q: Vec<_> = data.iter().flatten().cloned().collect();
        self.gmt.m2_modes(q.as_slice());
    }
}
impl<T: SegmentWiseSensor> Read<M2RigidBodyMotions> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(id, v)| {
            let (t_xyz, r_xyz) = v.split_at(3);
            self.gmt.m2_segment_state((id + 1) as i32, t_xyz, r_xyz);
        });
    }
}
impl<T: SegmentWiseSensor> Read<M2ASMReferenceBodyNodes> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2ASMReferenceBodyNodes>) {
        data.chunks(6).enumerate().for_each(|(id, v)| {
            let (t_xyz, r_xyz) = v.split_at(3);
            self.gmt.m2_segment_state((id + 1) as i32, t_xyz, r_xyz);
        });
    }
}
