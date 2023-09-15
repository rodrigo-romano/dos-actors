//! # M1 & M2 Rigid Body Motions to Linear Optical Model
//!
//! Transforms M1 and M2 rigid body motions to optical metrics
//! (tip-tilt, segment piston and  segment tip-tilt) using
//! linear optical sensitivity matrices

use std::{io::Read, sync::Arc};

use flate2::bufread::GzDecoder;
use gmt_dos_clients_io::{
    gmt_m1::M1RigidBodyMotions,
    gmt_m2::M2RigidBodyMotions,
    optics::{SegmentPiston, SegmentTipTilt, TipTilt},
};
use gmt_lom::{LinearOpticalModelError, LOM};
use interface::{self, Data, Size, Units, Update, Write};

/// M1 & M2 Rigid Body Motions to Linear Optical Model
#[derive(Debug)]
pub struct RigidBodyMotionsToLinearOpticalModel {
    lom: LOM,
    m1_rbm: Arc<Vec<f64>>,
    m2_rbm: Arc<Vec<f64>>,
}
impl RigidBodyMotionsToLinearOpticalModel {
    pub fn new() -> std::result::Result<Self, LinearOpticalModelError> {
        let sens = include_bytes!("optical_sensitivities.rs.bin.gz");
        let mut gz = GzDecoder::new(sens.as_slice());
        let mut bytes = vec![];
        gz.read_to_end(&mut bytes)?;
        Ok(Self {
            lom: LOM::try_from(bytes.as_slice())?,
            m1_rbm: Arc::new(vec![0f64; 42]),
            m2_rbm: Arc::new(vec![0f64; 42]),
        })
    }
}

impl Units for RigidBodyMotionsToLinearOpticalModel {}

impl Update for RigidBodyMotionsToLinearOpticalModel {
    fn update(&mut self) {
        self.lom.rbm = vec![(self.m1_rbm.as_slice(), self.m2_rbm.as_slice())]
            .into_iter()
            .collect();
    }
}
impl interface::Read<M1RigidBodyMotions> for RigidBodyMotionsToLinearOpticalModel {
    fn read(&mut self, data: Data<M1RigidBodyMotions>) {
        self.m1_rbm = data.as_arc();
    }
}
impl interface::Read<M2RigidBodyMotions> for RigidBodyMotionsToLinearOpticalModel {
    fn read(&mut self, data: Data<M2RigidBodyMotions>) {
        self.m2_rbm = data.as_arc();
    }
}
impl Write<TipTilt> for RigidBodyMotionsToLinearOpticalModel {
    fn write(&mut self) -> Option<Data<TipTilt>> {
        Some(Data::new(self.lom.tiptilt().into()))
    }
}
impl Write<SegmentTipTilt> for RigidBodyMotionsToLinearOpticalModel {
    fn write(&mut self) -> Option<Data<SegmentTipTilt>> {
        Some(Data::new(self.lom.segment_tiptilt().into()))
    }
}
impl Write<SegmentPiston> for RigidBodyMotionsToLinearOpticalModel {
    fn write(&mut self) -> Option<Data<SegmentPiston>> {
        Some(Data::new(self.lom.segment_piston().into()))
    }
}

impl Size<TipTilt> for RigidBodyMotionsToLinearOpticalModel {
    fn len(&self) -> usize {
        2
    }
}
impl Size<SegmentTipTilt> for RigidBodyMotionsToLinearOpticalModel {
    fn len(&self) -> usize {
        14
    }
}
impl Size<SegmentPiston> for RigidBodyMotionsToLinearOpticalModel {
    fn len(&self) -> usize {
        7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m1_segment_rxy() {
        let mut rbm2lom = RigidBodyMotionsToLinearOpticalModel::new().unwrap();
        let s = 1e-6;
        let m1_rbm: Vec<_> = (0..7)
            .flat_map(|_| {
                let mut v = vec![0f64; 6];
                v[3] = s;
                v
            })
            .collect();
        <RigidBodyMotionsToLinearOpticalModel as interface::Read<M1RigidBodyMotions>>::read(
            &mut rbm2lom,
            m1_rbm.into(),
        );
        rbm2lom.update();
        let mut stt: Vec<f64> =
            <RigidBodyMotionsToLinearOpticalModel as Write<SegmentTipTilt>>::write(&mut rbm2lom)
                .unwrap()
                .into();
        stt.iter_mut().for_each(|x| *x /= s);
        let stt_mag: Vec<_> = stt[..7]
            .iter()
            .zip(&stt[7..])
            .map(|(x, y)| x.hypot(*y))
            .collect();
        println!("STT: {:.2?}", stt_mag);
    }

    #[test]
    fn m2_segment_rxy() {
        let mut rbm2lom = RigidBodyMotionsToLinearOpticalModel::new().unwrap();
        let s = 1e-6;
        let m1_rbm: Vec<_> = (0..7)
            .flat_map(|_| {
                let mut v = vec![0f64; 6];
                v[4] = s;
                v
            })
            .collect();
        <RigidBodyMotionsToLinearOpticalModel as interface::Read<M2RigidBodyMotions>>::read(
            &mut rbm2lom,
            m1_rbm.into(),
        );
        rbm2lom.update();
        let mut stt: Vec<f64> =
            <RigidBodyMotionsToLinearOpticalModel as Write<SegmentTipTilt>>::write(&mut rbm2lom)
                .unwrap()
                .into();
        stt.iter_mut().for_each(|x| *x /= s);
        let stt_mag: Vec<_> = stt[..7]
            .iter()
            .zip(&stt[7..])
            .map(|(x, y)| x.hypot(*y))
            .collect();
        println!("STT: {:.2?}", stt_mag);
    }

    #[test]
    fn m1_segment_txy() {
        let mut rbm2lom = RigidBodyMotionsToLinearOpticalModel::new().unwrap();
        let s = 1e-6;
        let m1_rbm: Vec<_> = (0..7)
            .flat_map(|_| {
                let mut v = vec![0f64; 6];
                v[0] = s;
                v
            })
            .collect();
        <RigidBodyMotionsToLinearOpticalModel as interface::Read<M1RigidBodyMotions>>::read(
            &mut rbm2lom,
            m1_rbm.into(),
        );
        rbm2lom.update();
        let mut stt: Vec<f64> =
            <RigidBodyMotionsToLinearOpticalModel as Write<SegmentTipTilt>>::write(&mut rbm2lom)
                .unwrap()
                .into();
        stt.iter_mut().for_each(|x| *x /= s);
        let stt_mag: Vec<_> = stt[..7]
            .iter()
            .zip(&stt[7..])
            .map(|(x, y)| x.hypot(*y))
            .collect();
        println!("STT: {:.3?}", stt_mag);
    }
}
