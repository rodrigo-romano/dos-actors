use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crseo::{
    Atmosphere, AtmosphereBuilder, Builder, CrseoError, Gmt, GmtBuilder, Source, SourceBuilder,
};
use gmt_dos_clients_domeseeing::{DomeSeeing, DomeSeeingError};
use gmt_dos_clients_io::{
    gmt_m1::{segment::RBM, M1ModeShapes, M1RigidBodyMotions},
    gmt_m2::{asm::segment::FaceSheetFigure, M2RigidBodyMotions},
    optics::{M2modes, SegmentPiston, SegmentTipTilt, SegmentWfeRms, Wavefront, WfeRms},
};
use interface::{Data, Read, Size, TimerMarker, Units, Update, Write};

use super::GuideStar;

#[derive(Debug, thiserror::Error)]
pub enum LittleOpticalModelError {
    #[error(transparent)]
    CRSEO(#[from] CrseoError),
    #[error(transparent)]
    DomeSeeing(#[from] DomeSeeingError),
}

pub struct OpticalModel {
    pub gmt: Gmt,
    pub src: Arc<Mutex<Source>>,
    pub atm: Option<Atmosphere>,
    dome_seeing: Option<DomeSeeing>,
    pub tau: f64,
}
impl OpticalModel {
    pub fn builder() -> LittleOpticalModelBuilder {
        Default::default()
    }
}

impl Units for OpticalModel {}

#[derive(Debug, Default)]
pub struct LittleOpticalModelBuilder {
    gmt_builder: GmtBuilder,
    src_builder: SourceBuilder,
    atm_builder: Option<AtmosphereBuilder>,
    dome_seeing: Option<(PathBuf, usize)>,
    sampling_frequency: Option<f64>,
}
impl LittleOpticalModelBuilder {
    pub fn gmt(self, gmt_builder: GmtBuilder) -> Self {
        Self {
            gmt_builder,
            ..self
        }
    }
    pub fn source(self, src_builder: SourceBuilder) -> Self {
        Self {
            src_builder,
            ..self
        }
    }
    pub fn atmosphere(self, atm_builder: AtmosphereBuilder) -> Self {
        Self {
            atm_builder: Some(atm_builder),
            ..self
        }
    }
    pub fn dome_seeing<P: AsRef<Path>>(mut self, path: P, upsampling: usize) -> Self {
        self.dome_seeing = Some((path.as_ref().to_owned(), upsampling));
        self
    }
    pub fn sampling_frequency(self, sampling_frequency: f64) -> Self {
        Self {
            sampling_frequency: Some(sampling_frequency),
            ..self
        }
    }
    pub fn build(self) -> Result<OpticalModel, LittleOpticalModelError> {
        let gmt = self.gmt_builder.build()?;
        let src = self.src_builder.build()?;
        let atm = if let Some(atm_builder) = self.atm_builder {
            Some(atm_builder.build()?)
        } else {
            None
        };
        let dome_seeing = if let Some((path, upsampling)) = self.dome_seeing {
            Some(DomeSeeing::new(path.to_str().unwrap(), upsampling, None)?)
        } else {
            None
        };
        Ok(OpticalModel {
            gmt,
            src: Arc::new(Mutex::new(src)),
            atm,
            dome_seeing,
            tau: self.sampling_frequency.map_or_else(|| 0f64, |x| x.recip()),
        })
    }
}
impl TimerMarker for OpticalModel {}
impl Update for OpticalModel {
    fn update(&mut self) {
        let src = &mut (*self.src.lock().unwrap());
        src.through(&mut self.gmt).xpupil();
        if let Some(atm) = &mut self.atm {
            atm.secs += self.tau;
            src.through(atm);
        }
        if let Some(dome_seeing) = &mut self.dome_seeing {
            src.add_same(dome_seeing.next().unwrap().as_slice());
        }
    }
}

impl Write<GuideStar> for OpticalModel {
    fn write(&mut self) -> Option<Data<GuideStar>> {
        Some(Data::new(self.src.clone()))
    }
}

impl Read<M2modes> for OpticalModel {
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
impl<const ID: u8> Read<FaceSheetFigure<ID>> for OpticalModel {
    fn read(&mut self, data: Data<FaceSheetFigure<ID>>) {
        self.gmt.m2_segment_modes(ID, &data);
    }
}
impl Read<M1RigidBodyMotions> for OpticalModel {
    fn read(&mut self, data: Data<M1RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(id, v)| {
            let (t_xyz, r_xyz) = v.split_at(3);
            self.gmt.m1_segment_state((id + 1) as i32, t_xyz, r_xyz);
        });
    }
}
impl Read<M2RigidBodyMotions> for OpticalModel {
    fn read(&mut self, data: Data<M2RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(id, v)| {
            let (t_xyz, r_xyz) = v.split_at(3);
            self.gmt.m2_segment_state((id + 1) as i32, t_xyz, r_xyz);
        });
    }
}
impl<const ID: u8> Read<RBM<ID>> for OpticalModel {
    fn read(&mut self, data: Data<RBM<ID>>) {
        let (t_xyz, r_xyz) = data.split_at(3);
        self.gmt.m1_segment_state(ID as i32, &t_xyz, &r_xyz);
    }
}
impl Size<WfeRms> for OpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        src.size as usize
    }
}

impl Write<WfeRms> for OpticalModel {
    fn write(&mut self) -> Option<Data<WfeRms>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.wfe_rms()))
    }
}

impl Size<SegmentWfeRms> for OpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        (src.size as usize) * 7
    }
}
impl Write<SegmentWfeRms> for OpticalModel {
    fn write(&mut self) -> Option<Data<SegmentWfeRms>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.segment_wfe_rms()))
    }
}

impl Size<SegmentPiston> for OpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        (src.size as usize) * 7
    }
}
impl Write<SegmentPiston> for OpticalModel {
    fn write(&mut self) -> Option<Data<SegmentPiston>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.segment_piston()))
    }
}

impl Size<SegmentTipTilt> for OpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        (src.size as usize) * 7 * 2
    }
}
impl Write<SegmentTipTilt> for OpticalModel {
    fn write(&mut self) -> Option<Data<SegmentTipTilt>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.segment_gradients()))
    }
}

impl Size<Wavefront> for OpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        src.pupil_sampling().pow(2)
    }
}
impl Write<Wavefront> for OpticalModel {
    fn write(&mut self) -> Option<Data<Wavefront>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.phase().clone()))
    }
}

#[derive(interface::UID)]
#[uid(data = (Vec<f32>,Vec<bool>))]
pub enum GmtWavefront {}

impl Write<GmtWavefront> for OpticalModel {
    fn write(&mut self) -> Option<Data<GmtWavefront>> {
        let src = &mut (self.src.lock().unwrap());
        let amplitude: Vec<_> = src.amplitude().into_iter().map(|a| a > 0.).collect();
        let phase = src.phase();
        let phase: Vec<_> = amplitude
            .iter()
            .zip(phase)
            .filter(|(&a, _)| a)
            .map(|(_, &p)| p)
            .collect();
        Some(Data::new((phase, amplitude)))
    }
}

impl Read<M1ModeShapes> for OpticalModel {
    fn read(&mut self, data: Data<M1ModeShapes>) {
        self.gmt.m1_modes(&*data);
    }
}

impl Size<M1ModeShapes> for OpticalModel {
    fn len(&self) -> usize {
        self.gmt.m1_n_mode * 7
    }
}
