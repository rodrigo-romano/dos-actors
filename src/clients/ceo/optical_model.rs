use crate::{
    io::{Data, Read, Write},
    Update,
};
use crseo::{
    pssn::TelescopeError, Atmosphere, Builder, Diffractive, Geometric, Gmt, PSSn, Propagation,
    ShackHartmann, Source, WavefrontSensor, WavefrontSensorBuilder, ATMOSPHERE, GMT, PSSN, SH24,
    SHACKHARTMANN, SOURCE,
};
use nalgebra as na;
use std::{marker::PhantomData, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum CeoError {
    #[error("CEO building failed")]
    CEO(#[from] crseo::CrseoError),
}
pub type Result<T> = std::result::Result<T, CeoError>;

pub struct SensorBuilder<S = ShackHartmann<Geometric>, B = SHACKHARTMANN<Geometric>>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    sensor: B,
}
impl<S, B> SensorBuilder<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    pub fn new(sensor: B) -> Self {
        Self { sensor }
    }
    pub fn build(self) -> Result<S> {
        let sensor = self.sensor.build()?;
        Ok(sensor)
    }
}

/// GMT optical model builder
pub struct OpticalModelBuilder<S = ShackHartmann<Geometric>, B = SHACKHARTMANN<Geometric>>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    gmt: GMT,
    src: SOURCE,
    atm: Option<ATMOSPHERE>,
    sensor: Option<SensorBuilder<S, B>>,
    pssn: Option<PSSN<TelescopeError>>,
    flux_threshold: f64,
}
impl<S, B> Default for OpticalModelBuilder<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn default() -> Self {
        Self {
            gmt: GMT::default(),
            src: SOURCE::default(),
            atm: None,
            sensor: None,
            pssn: None,
            flux_threshold: 0.8,
        }
    }
}
impl<S, B> OpticalModelBuilder<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    /// Creates a new GMT optical model
    ///
    /// Creates a default builder based on the default parameters for [GMT] and [SOURCE]
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the GMT builder
    pub fn gmt(self, gmt: GMT) -> Self {
        Self { gmt, ..self }
    }
    /// Sets the `Source` builder
    pub fn source(self, src: SOURCE) -> Self {
        Self { src, ..self }
    }
    /// Sets the `sensor` builder
    pub fn sensor_builder(self, sensor_builder: SensorBuilder<S, B>) -> Self {
        Self {
            sensor: Some(sensor_builder),
            ..self
        }
    }
    /// Sets the [atmosphere](ATMOSPHERE) builder
    pub fn atmosphere(self, atm: ATMOSPHERE) -> Self {
        Self {
            atm: Some(atm),
            ..self
        }
    }
    /// Builds a new GMT optical model
    ///
    /// If there is `Some` sensor, it is initialized.
    pub fn build(self) -> Result<OpticalModel<S, B>> {
        if let Some(sensor_builder) = self.sensor {
            let mut gmt = self.gmt.build()?;
            let mut src = sensor_builder
                .sensor
                .guide_stars(Some(self.src.clone()))
                .build()?;
            let mut sensor = sensor_builder.sensor.build()?;
            gmt.reset();
            src.through(&mut gmt).xpupil();
            sensor.calibrate(&mut src, self.flux_threshold);
            Ok(OpticalModel {
                gmt,
                src,
                sensor: Some(sensor),
                atm: match self.atm {
                    Some(atm) => Some(atm.build()?),
                    None => None,
                },
                pssn: match self.pssn {
                    Some(pssn) => Some(pssn.source(&(self.src.build()?)).build()?),
                    None => None,
                },
                sensor_fn: SensorFn::None,
                builder: PhantomData,
            })
        } else {
            let gmt = self.gmt.build()?;
            let src = self.src.clone().build()?;
            Ok(OpticalModel {
                gmt,
                src,
                sensor: None,
                atm: match self.atm {
                    Some(atm) => Some(atm.build()?),
                    None => None,
                },
                pssn: match self.pssn {
                    Some(pssn) => Some(pssn.source(&(self.src.build()?)).build()?),
                    None => None,
                },
                sensor_fn: SensorFn::None,
                builder: PhantomData,
            })
        }
    }
}
pub enum SensorFn {
    None,
    Fn(Box<dyn Fn(Vec<f64>) -> Vec<f64> + Send>),
    Matrix(na::DMatrix<f64>),
}
/// GMT Optical Model
pub struct OpticalModel<S = ShackHartmann<Geometric>, B = SHACKHARTMANN<Geometric>>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    pub gmt: Gmt,
    pub src: Source,
    pub sensor: Option<S>,
    pub atm: Option<Atmosphere>,
    pub pssn: Option<PSSn<TelescopeError>>,
    pub sensor_fn: SensorFn,
    builder: PhantomData<B>,
}
impl<S, B> OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    pub fn builder() -> OpticalModelBuilder<S, B> {
        OpticalModelBuilder::new()
    }
    pub fn sensor_matrix_transform(&mut self, mat: na::DMatrix<f64>) -> &mut Self {
        self.sensor_fn = SensorFn::Matrix(mat);
        self
    }
}

impl<S, B> Update for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn update(&mut self) {
        self.src.through(&mut self.gmt).xpupil();
        if let Some(atm) = &mut self.atm {
            self.src.through(atm);
        }
        if let Some(sensor) = &mut self.sensor {
            self.src.through(sensor);
        }
        if let Some(pssn) = &mut self.pssn {
            self.src.through(pssn);
        }
    }
}

#[cfg(feature = "fem")]
impl<S, B> Read<Vec<f64>, fem::fem_io::OSSM1Lcl> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn read(&mut self, data: Arc<Data<Vec<f64>, fem::fem_io::OSSM1Lcl>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
#[cfg(feature = "fem")]
impl<S, B> Read<Vec<f64>, fem::fem_io::MCM2Lcl6D> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn read(&mut self, data: Arc<Data<Vec<f64>, fem::fem_io::MCM2Lcl6D>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m2_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
impl<S, B> Write<Vec<f64>, super::WfeRms> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::WfeRms>>> {
        Some(Arc::new(Data::new(self.src.wfe_rms())))
    }
}
impl<S, B> Write<Vec<f64>, super::SegmentWfeRms> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentWfeRms>>> {
        Some(Arc::new(Data::new(self.src.segment_wfe_rms())))
    }
}
impl<S, B> Write<Vec<f64>, super::SegmentPiston> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentPiston>>> {
        Some(Arc::new(Data::new(self.src.segment_piston())))
    }
}
impl<S, B> Write<Vec<f64>, super::SegmentGradients> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentGradients>>> {
        Some(Arc::new(Data::new(self.src.segment_gradients())))
    }
}
impl<S, B> Write<Vec<f64>, super::PSSn> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S>,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::PSSn>>> {
        self.pssn.as_mut().map(|pssn| {
            Arc::new(Data::new(
                pssn.peek()
                    .estimates
                    .iter()
                    .cloned()
                    .map(|x| x as f64)
                    .collect(),
            ))
        })
    }
}
impl Write<Vec<f64>, super::SensorData>
    for OpticalModel<ShackHartmann<Diffractive>, SHACKHARTMANN<Diffractive>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.readout().process();
            let data: Vec<f64> = sensor.get_data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(
                    data.into_iter().map(|x| x as f64).collect(),
                ))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data
                    .into_iter()
                    .map(|x| x as f64)
                    .collect())))),
                SensorFn::Matrix(mat) => {
                    let u: Vec<_> = data.into_iter().map(|x| x as f64).collect();
                    let v = na::DVector::from_vec(u);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
impl Write<Vec<f64>, super::SensorData>
    for OpticalModel<ShackHartmann<Geometric>, SHACKHARTMANN<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.get_data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(
                    data.into_iter().map(|x| x as f64).collect(),
                ))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data
                    .into_iter()
                    .map(|x| x as f64)
                    .collect())))),
                SensorFn::Matrix(mat) => {
                    let u: Vec<_> = data.into_iter().map(|x| x as f64).collect();
                    let v = na::DVector::from_vec(u);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
impl Write<Vec<f64>, crate::clients::fsm::TTFB>
    for OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.get_data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(
                    data.into_iter().map(|x| x as f64).collect(),
                ))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data
                    .into_iter()
                    .map(|x| x as f64)
                    .collect())))),
                SensorFn::Matrix(mat) => {
                    let u: Vec<_> = data.into_iter().map(|x| x as f64).collect();
                    let v = na::DVector::from_vec(u);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
