use crate::{
    clients::dome_seeing::{DomeSeeing, DomeSeeingOpd},
    io::{Data, Read, Write},
    Size, Update,
};
use crseo::{
    cu,
    pssn::{AtmosphereTelescopeError, TelescopeError},
    Atmosphere, AtmosphereBuilder, Builder, Cu, Diffractive, Fwhm, Geometric, Gmt, GmtBuilder,
    PSSnBuilder, PSSnEstimates, ShackHartmannBuilder, Source, SourceBuilder, WavefrontSensor,
    WavefrontSensorBuilder,
};
use nalgebra as na;
use std::{ops::DerefMut, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum CeoError {
    #[error("CEO building failed")]
    CEO(#[from] crseo::CrseoError),
}
pub type Result<T> = std::result::Result<T, CeoError>;

/// Shack-Hartmann wavefront sensor type: [Diffractive] or [Geometric]
#[derive(PartialEq, Clone)]
pub enum ShackHartmannOptions {
    Diffractive(ShackHartmannBuilder<Diffractive>),
    Geometric(ShackHartmannBuilder<Geometric>),
}
/// PSSn model
#[derive(PartialEq, Clone)]
pub enum PSSnOptions {
    Telescope(PSSnBuilder<TelescopeError>),
    AtmosphereTelescope(PSSnBuilder<AtmosphereTelescopeError>),
}
type Cuf32 = Cu<cu::Single>;
/// Options for [OpticalModelBuilder]
#[derive(Clone)]
pub enum OpticalModelOptions {
    Atmosphere {
        builder: AtmosphereBuilder,
        time_step: f64,
    },
    ShackHartmann {
        options: ShackHartmannOptions,
        flux_threshold: f64,
    },
    DomeSeeing {
        cfd_case: String,
        upsampling_rate: usize,
    },
    StaticAberration(Cuf32),
    PSSn(PSSnOptions),
}

/// GmtBuilder optical model builder
pub struct OpticalModelBuilder {
    gmt: GmtBuilder,
    src: SourceBuilder,
    options: Option<Vec<OpticalModelOptions>>,
}
impl Default for OpticalModelBuilder {
    fn default() -> Self {
        Self {
            gmt: GmtBuilder::default(),
            src: SourceBuilder::default(),
            options: None,
        }
    }
}

pub trait SensorBuilder: WavefrontSensorBuilder + Builder + Clone {
    fn build(
        self,
        gmt_builder: GmtBuilder,
        src_builder: SourceBuilder,
        threshold: f64,
    ) -> Result<Box<dyn WavefrontSensor>>;
}

impl OpticalModelBuilder {
    /// Creates a new GmtBuilder optical model
    ///
    /// Creates a default builder based on the default parameters for [GmtBuilder] and [SourceBuilder]
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the GmtBuilder builder
    pub fn gmt(self, gmt: GmtBuilder) -> Self {
        Self { gmt, ..self }
    }
    /// Sets the `Source` builder
    pub fn source(self, src: SourceBuilder) -> Self {
        Self { src, ..self }
    }
    /// Sets [OpticalModel] [options](OpticalModelOptions)
    pub fn options(self, options: Vec<OpticalModelOptions>) -> Self {
        Self {
            options: Some(options),
            ..self
        }
    }
    /// Builds a new GmtBuilder optical model
    ///
    /// If there is `Some` sensor, it is initialized.

    pub fn build(self) -> Result<OpticalModel> {
        let gmt = self.gmt.clone().build()?;
        let src = self.src.clone().build()?;
        let mut optical_model = OpticalModel {
            gmt,
            src,
            sensor: None,
            atm: None,
            dome_seeing: None,
            static_aberration: None,
            pssn: None,
            sensor_fn: SensorFn::None,
            frame: None,
            tau: 0f64,
        };
        if let Some(options) = self.options {
            options.into_iter().for_each(|option| match option {
                OpticalModelOptions::PSSn(PSSnOptions::Telescope(pssn_builder)) => {
                    optical_model.pssn = pssn_builder
                        .source(self.src.clone())
                        .build()
                        .ok()
                        .map(|x| Box::new(x) as Box<dyn PSSnEstimates>);
                }
                OpticalModelOptions::PSSn(PSSnOptions::AtmosphereTelescope(pssn_builder)) => {
                    optical_model.pssn = pssn_builder
                        .source(self.src.clone())
                        .build()
                        .ok()
                        .map(|x| Box::new(x) as Box<dyn PSSnEstimates>);
                }
                OpticalModelOptions::Atmosphere { builder, time_step } => {
                    optical_model.atm = builder.build().ok();
                    optical_model.tau = time_step;
                }
                OpticalModelOptions::ShackHartmann {
                    options,
                    flux_threshold,
                } => match options {
                    ShackHartmannOptions::Diffractive(sensor_builder) => {
                        optical_model.src = sensor_builder
                            .guide_stars(Some(self.src.clone()))
                            .build()
                            .unwrap();
                        optical_model.sensor = SensorBuilder::build(
                            sensor_builder,
                            self.gmt.clone(),
                            self.src.clone(),
                            flux_threshold,
                        )
                        .ok();
                    }
                    ShackHartmannOptions::Geometric(sensor_builder) => {
                        optical_model.src = sensor_builder
                            .guide_stars(Some(self.src.clone()))
                            .build()
                            .unwrap();
                        optical_model.sensor = SensorBuilder::build(
                            sensor_builder,
                            self.gmt.clone(),
                            self.src.clone(),
                            flux_threshold,
                        )
                        .ok();
                    }
                },
                OpticalModelOptions::DomeSeeing {
                    cfd_case,
                    upsampling_rate,
                } => {
                    optical_model.dome_seeing =
                        DomeSeeing::new(cfd_case, upsampling_rate, None).ok();
                }
                OpticalModelOptions::StaticAberration(phase) => {
                    optical_model.static_aberration = Some(phase);
                }
            });
        }
        if let Some(dome_seeing) = optical_model.dome_seeing.as_ref() {
            let n_ds = <DomeSeeing as Size<DomeSeeingOpd>>::len(dome_seeing);
            let n_src = optical_model.src.pupil_sampling.pow(2) as usize;
            assert_eq!(
                n_ds, n_src,
                "the sizes of dome seeing and source wavefront do not match, {n_ds} versus {n_src}"
            );
        }
        if let Some(static_aberration) = optical_model.static_aberration.as_ref() {
            let n_sa = static_aberration.size();
            let n_src = optical_model.src.pupil_sampling.pow(2) as usize;
            assert_eq!(
                n_sa, n_src,
                "the sizes of static aberration and source wavefront do not match, {n_sa} versus {n_src}"
            );
        }
        Ok(optical_model)
    }
}
pub enum SensorFn {
    None,
    Fn(Box<dyn Fn(Vec<f64>) -> Vec<f64> + Send>),
    Matrix(na::DMatrix<f64>),
}
impl Default for SensorFn {
    fn default() -> Self {
        SensorFn::None
    }
}
/// GmtBuilder Optical Model
pub struct OpticalModel {
    pub gmt: Gmt,
    pub src: Source,
    pub sensor: Option<Box<dyn WavefrontSensor>>,
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
            self.src.add_same(&mut dome_seeing.next().unwrap().into());
        }
        if let Some(static_aberration) = &mut self.static_aberration {
            self.src.add_same(static_aberration);
        }
        if let Some(sensor) = &mut self.sensor {
            //self.src.through(sensor);
            sensor.deref_mut().propagate(&mut self.src);
        }
        if let Some(pssn) = &mut self.pssn {
            self.src.through(pssn);
        }
    }
}

impl crate::clients::TimerMarker for OpticalModel {}

#[cfg(feature = "crseo")]
impl Read<crseo::gmt::SegmentsDof, super::GmtState> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::GmtState>>) {
        if let Err(e) = &data.apply_to(&mut self.gmt) {
            crate::print_error("Failed applying GmtBuilder state", e);
        }
    }
}
impl Read<super::M1rbm> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::M1rbm>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
impl Read<super::M1modes> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::M1modes>>) {
        self.gmt.m1_modes(&data);
    }
}
impl Write<super::M1modes> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::M1modes>>> {
        Some(Arc::new(Data::new(self.gmt.a1.clone())))
    }
}
impl Read<super::M2rbm> for OpticalModel {
    fn read(&mut self, data: Arc<Data<super::M2rbm>>) {
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
#[cfg(feature = "fem")]
impl Read<fem::fem_io::OSSM1Lcl> for OpticalModel {
    fn read(&mut self, data: Arc<Data<fem::fem_io::OSSM1Lcl>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
#[cfg(feature = "fem")]
impl Read<fem::fem_io::MCM2Lcl6D> for OpticalModel {
    fn read(&mut self, data: Arc<Data<fem::fem_io::MCM2Lcl6D>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m2_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
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
