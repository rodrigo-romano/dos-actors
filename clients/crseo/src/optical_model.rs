use crseo::{
    cu, wavefrontsensor::Calibration, Atmosphere, Cu, Fwhm, Gmt, PSSnEstimates, Source,
    WavefrontSensor,
};
use gmt_dos_clients::interface::{Data, Read, Update, Write};
use gmt_dos_clients_domeseeing::DomeSeeing;
use nalgebra as na;
use std::{fmt::Debug, ops::DerefMut, sync::Arc};

mod builder;
pub use builder::OpticalModelBuilder;
mod options;
pub use options::{OpticalModelOptions, PSSnOptions, ShackHartmannOptions};

type Cuf32 = Cu<cu::Single>;

/// Sensor data transform operator
pub enum SensorFn {
    None,
    Fn(Box<dyn Fn(Vec<f64>) -> Vec<f64> + Send>),
    Matrix(na::DMatrix<f64>),
    Calibration(Calibration),
}
impl Default for SensorFn {
    fn default() -> Self {
        SensorFn::None
    }
}
impl Debug for SensorFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Fn(_arg0) => f.debug_tuple("Fn").finish(),
            Self::Matrix(arg0) => f.debug_tuple("Matrix").field(arg0).finish(),
            Self::Calibration(arg0) => f.debug_tuple("Calibration").field(arg0).finish(),
        }
    }
}
impl From<na::DMatrix<f64>> for SensorFn {
    fn from(value: na::DMatrix<f64>) -> Self {
        SensorFn::Matrix(value)
    }
}
impl From<Calibration> for SensorFn {
    fn from(value: Calibration) -> Self {
        SensorFn::Calibration(value)
    }
}
impl From<Box<dyn Fn(Vec<f64>) -> Vec<f64> + Send>> for SensorFn {
    fn from(value: Box<dyn Fn(Vec<f64>) -> Vec<f64> + Send>) -> Self {
        SensorFn::Fn(value)
    }
}

/// GmtBuilder Optical Model
pub struct OpticalModel {
    pub gmt: Gmt,
    pub src: Source,
    pub sensor: Option<Box<dyn WavefrontSensor>>,
    pub segment_wise_sensor: Option<Box<dyn WavefrontSensor>>,
    pub atm: Option<Atmosphere>,
    pub dome_seeing: Option<DomeSeeing>,
    pub static_aberration: Option<Cuf32>,
    pub pssn: Option<Box<dyn PSSnEstimates>>,
    pub sensor_fn: SensorFn,
    pub(crate) frame: Option<Vec<f32>>,
    tau: f64,
}
impl OpticalModel {
    pub fn builder() -> OpticalModelBuilder {
        OpticalModelBuilder::new()
    }
    pub fn sensor_matrix_transform(&mut self, mat: na::DMatrix<f64>) -> &mut Self {
        self.sensor_fn = SensorFn::Matrix(mat);
        self
    }
}

impl Update for OpticalModel {
    fn update(&mut self) {
        self.src.through(&mut self.gmt).xpupil();
        if let Some(atm) = &mut self.atm {
            atm.secs += self.tau;
            self.src.through(atm);
        }
        if let Some(dome_seeing) = &mut self.dome_seeing {
            self.src.add_same(dome_seeing.next().unwrap().as_slice());
        }
        if let Some(static_aberration) = &self.static_aberration {
            self.src.add_same(&[static_aberration]);
        }
        if let Some(sensor) = &mut self.sensor {
            //self.src.through(sensor);
            sensor.deref_mut().propagate(&mut self.src);
        }
        if let Some(sensor) = &mut self.segment_wise_sensor {
            (*sensor).propagate(&mut self.src);
        }
        if let Some(pssn) = &mut self.pssn {
            self.src.through(pssn);
        }
    }
}

impl gmt_dos_clients::interface::TimerMarker for OpticalModel {}

#[cfg(feature = "crseo")]
impl Read<super::GmtState> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::GmtState>>) {
        if let Err(e) = &data.apply_to(&mut self.gmt) {
            crate::print_error("Failed applying GmtBuilder state", e);
        }
    }
}
impl Read<super::M1RigidBodyMotions> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::M1RigidBodyMotions>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
impl Read<super::M1ModeShapes> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::M1ModeShapes>>) {
        self.gmt.m1_modes(&data);
    }
}
impl Write<super::M1ModeShapes> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::M1ModeShapes>>> {
        Some(Arc::new(Data::new(self.gmt.a1.clone())))
    }
}
impl Read<super::M2RigidBodyMotions> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::M2RigidBodyMotions>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m2_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
impl Read<super::M2rxy> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::M2rxy>>) {
        let t_xyz = vec![0f64; 3];
        let mut r_xyz = vec![0f64; 3];
        data.chunks(2).enumerate().for_each(|(sid0, v)| {
            r_xyz[0] = v[0];
            r_xyz[1] = v[1];
            self.gmt.m2_segment_state((sid0 + 1) as i32, &t_xyz, &r_xyz);
        });
    }
}
impl Read<super::M2modes> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::M2modes>>) {
        self.gmt.m2_modes(&data);
    }
}
impl Write<super::M2modes> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::M2modes>>> {
        Some(Arc::new(Data::new(self.gmt.a2.clone())))
    }
}
impl Write<super::WfeRms> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::WfeRms>>> {
        Some(Arc::new(Data::new(self.src.wfe_rms())))
    }
}
impl Write<super::Wavefront> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::Wavefront>>> {
        Some(Arc::new(Data::new(self.src.phase().to_vec())))
    }
}
impl Write<super::TipTilt> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::TipTilt>>> {
        Some(Arc::new(Data::new(self.src.gradients())))
    }
}
impl Write<super::SegmentWfe> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::SegmentWfe>>> {
        Some(Arc::new(Data::new(
            self.src
                .segment_wfe()
                .into_iter()
                .flat_map(|(p, s)| vec![p, s])
                .collect(),
        )))
    }
}
impl Write<super::SegmentWfeRms> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::SegmentWfeRms>>> {
        Some(Arc::new(Data::new(self.src.segment_wfe_rms())))
    }
}
impl Write<super::SegmentPiston> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::SegmentPiston>>> {
        Some(Arc::new(Data::new(self.src.segment_piston())))
    }
}
impl Write<super::SegmentGradients> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::SegmentGradients>>> {
        Some(Arc::new(Data::new(self.src.segment_gradients())))
    }
}
impl Write<super::SegmentTipTilt> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::SegmentTipTilt>>> {
        Some(Arc::new(Data::new(self.src.segment_gradients())))
    }
}
impl Write<super::PSSn> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::PSSn>>> {
        match self.pssn {
            Some(ref mut pssn) => Some(Arc::new(Data::new(pssn.estimates()))),
            None => panic!("PSSn is not declared for this optical model"),
        }
    }
}
impl Write<super::PSSnFwhm> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::PSSnFwhm>>> {
        match self.pssn {
            Some(ref mut pssn) => {
                let mut fwhm = Fwhm::new();
                fwhm.build(&mut self.src);
                let data: Vec<f64> = vec![pssn.estimates(), fwhm.from_complex_otf(&pssn.otf())]
                    .into_iter()
                    .flatten()
                    .collect();
                Some(Arc::new(Data::new(data)))
            }
            None => panic!("PSSn is not declared for this optical model"),
        }
    }
}

impl Read<super::PointingError> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::PointingError>>) {
        self.gmt.pointing_error = Some((**data).clone());
    }
}
