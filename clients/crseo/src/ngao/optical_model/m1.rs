use crseo::SegmentWiseSensor;
use gmt_dos_clients_io::gmt_m1::{segment::RBM, M1ModeShapes, M1RigidBodyMotions};
use interface::{Data, Read, Size};

use crate::OpticalModel;

impl<T: SegmentWiseSensor> Read<M1RigidBodyMotions> for OpticalModel<T> {
    fn read(&mut self, data: Data<M1RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(id, v)| {
            let (t_xyz, r_xyz) = v.split_at(3);
            self.gmt.m1_segment_state((id + 1) as i32, t_xyz, r_xyz);
        });
    }
}

impl<T: SegmentWiseSensor, const ID: u8> Read<RBM<ID>> for OpticalModel<T> {
    fn read(&mut self, data: Data<RBM<ID>>) {
        let (t_xyz, r_xyz) = data.split_at(3);
        self.gmt.m1_segment_state(ID as i32, &t_xyz, &r_xyz);
    }
}
impl<T: SegmentWiseSensor> Read<M1ModeShapes> for OpticalModel<T> {
    fn read(&mut self, data: Data<M1ModeShapes>) {
        self.gmt.m1_modes(&*data);
    }
}

impl<T: SegmentWiseSensor> Size<M1ModeShapes> for OpticalModel<T> {
    fn len(&self) -> usize {
        self.gmt.m1_n_mode * 7
    }
}
