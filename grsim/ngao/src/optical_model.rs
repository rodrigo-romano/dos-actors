use std::sync::{Arc, Mutex};

use crseo::{
    Atmosphere, AtmosphereBuilder, Builder, CrseoError, Gmt, GmtBuilder, Source, SourceBuilder,
};
use gmt_dos_clients::interface::{Data, Read, Size, TimerMarker, Update, Write};
use gmt_dos_clients_crseo::{M2modes, SegmentPiston, SegmentTipTilt, SegmentWfeRms, WfeRms};
use gmt_dos_clients_io::{
    gmt_m1::{segment::RBM, M1ModeShapes, M1RigidBodyMotions},
    gmt_m2::asm::segment::FaceSheetFigure,
};

use crate::GuideStar;

pub struct LittleOpticalModel {
    pub gmt: Gmt,
    pub src: Arc<Mutex<Source>>,
    pub atm: Option<Atmosphere>,
    pub tau: f64,
}
impl LittleOpticalModel {
    pub fn builder() -> LittleOpticalModelBuilder {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct LittleOpticalModelBuilder {
    gmt_builder: GmtBuilder,
    src_builder: SourceBuilder,
    atm_builder: Option<AtmosphereBuilder>,
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
    pub fn sampling_frequency(self, sampling_frequency: f64) -> Self {
        Self {
            sampling_frequency: Some(sampling_frequency),
            ..self
        }
    }
    pub fn build(self) -> Result<LittleOpticalModel, CrseoError> {
        let gmt = self.gmt_builder.build()?;
        let src = self.src_builder.build()?;
        let atm = if let Some(atm_builder) = self.atm_builder {
            Some(atm_builder.build()?)
        } else {
            None
        };
        Ok(LittleOpticalModel {
            gmt,
            src: Arc::new(Mutex::new(src)),
            atm,
            tau: self.sampling_frequency.map_or_else(|| 0f64, |x| x.recip()),
        })
    }
}
impl TimerMarker for LittleOpticalModel {}
impl Update for LittleOpticalModel {
    fn update(&mut self) {
        let src = &mut (*self.src.lock().unwrap());
        src.through(&mut self.gmt).xpupil();
        if let Some(atm) = &mut self.atm {
            atm.secs += self.tau;
            src.through(atm);
        }
    }
}

impl Write<GuideStar> for LittleOpticalModel {
    fn write(&mut self) -> Option<Data<GuideStar>> {
        Some(Data::new(self.src.clone()))
    }
}

impl Read<M2modes> for LittleOpticalModel {
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
impl<const ID: u8> Read<FaceSheetFigure<ID>> for LittleOpticalModel {
    fn read(&mut self, data: Data<FaceSheetFigure<ID>>) {
        self.gmt.m2_segment_modes(ID, &data);
    }
}
impl Read<M1RigidBodyMotions> for LittleOpticalModel {
    fn read(&mut self, data: Data<M1RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(id, v)| {
            let (t_xyz, r_xyz) = v.split_at(3);
            self.gmt.m1_segment_state((id + 1) as i32, t_xyz, r_xyz);
        });
    }
}
impl<const ID: u8> Read<RBM<ID>> for LittleOpticalModel {
    fn read(&mut self, data: Data<RBM<ID>>) {
        let (t_xyz, r_xyz) = data.split_at(3);
        self.gmt.m1_segment_state(ID as i32, &t_xyz, &r_xyz);
    }
}
impl Size<WfeRms> for LittleOpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        src.size as usize
    }
}
impl Write<WfeRms> for LittleOpticalModel {
    fn write(&mut self) -> Option<Data<WfeRms>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.wfe_rms()))
    }
}

impl Size<SegmentWfeRms> for LittleOpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        (src.size as usize) * 7
    }
}
impl Write<SegmentWfeRms> for LittleOpticalModel {
    fn write(&mut self) -> Option<Data<SegmentWfeRms>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.segment_wfe_rms()))
    }
}

impl Size<SegmentPiston> for LittleOpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        (src.size as usize) * 7
    }
}
impl Write<SegmentPiston> for LittleOpticalModel {
    fn write(&mut self) -> Option<Data<SegmentPiston>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.segment_piston()))
    }
}

impl Size<SegmentTipTilt> for LittleOpticalModel {
    fn len(&self) -> usize {
        let src = &mut (self.src.lock().unwrap());
        (src.size as usize) * 7 * 2
    }
}
impl Write<SegmentTipTilt> for LittleOpticalModel {
    fn write(&mut self) -> Option<Data<SegmentTipTilt>> {
        let src = &mut (self.src.lock().unwrap());
        Some(Data::new(src.segment_gradients()))
    }
}

impl Read<M1ModeShapes> for LittleOpticalModel {
    fn read(&mut self, data: Data<M1ModeShapes>) {
        self.gmt.m1_modes(&*data);
    }
}

impl Size<M1ModeShapes> for LittleOpticalModel {
    fn len(&self) -> usize {
        self.gmt.m1_n_mode * 7
    }
}
