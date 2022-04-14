use crate::{
    io::{Data, Read, Write},
    Update,
};
use crseo::{
    pssn::TelescopeError, Atmosphere, Builder, Diffractive, Geometric, Gmt, PSSn, Propagation,
    ShackHartmann, ShackHartmannBuilder, Source, WavefrontSensor, WavefrontSensorBuilder,
    ATMOSPHERE, GMT, PSSN, SH24, SH48, SOURCE,
};
use nalgebra as na;
use std::{marker::PhantomData, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum CeoError {
    #[error("CEO building failed")]
    CEO(#[from] crseo::CrseoError),
}
pub type Result<T> = std::result::Result<T, CeoError>;

/// GMT optical model builder
pub struct OpticalModelBuilder<S = ShackHartmann<Geometric>, B = ShackHartmannBuilder<Geometric>>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    gmt: GMT,
    src: SOURCE,
    atm: Option<ATMOSPHERE>,
    sensor: Option<B>,
    pssn: Option<PSSN<TelescopeError>>,
    flux_threshold: f64,
}
impl<S, B> Default for OpticalModelBuilder<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn default() -> Self {
        Self {
            gmt: GMT::default(),
            src: SOURCE::default(),
            atm: None,
            sensor: None,
            pssn: None,
            flux_threshold: 0.1,
        }
    }
}

pub trait SensorBuilder {
    type Sensor;
    fn build(self, gmt_builder: GMT, src_builder: SOURCE, threshold: f64) -> Result<Self::Sensor>;
}

impl SensorBuilder for ShackHartmannBuilder<Geometric> {
    type Sensor = ShackHartmann<Geometric>;
    fn build(self, gmt_builder: GMT, src_builder: SOURCE, threshold: f64) -> Result<Self::Sensor> {
        let mut src = self.guide_stars(Some(src_builder)).build()?;
        let n_side_lenslet = self.lenslet_array.0;
        let n = n_side_lenslet.pow(2) * self.n_sensor;
        let mut valid_lenslets: Vec<i32> = (1..=7).fold(vec![0i32; n as usize], |mut a, sid| {
            let mut gmt = gmt_builder.clone().build().unwrap();
            src.reset();
            src.through(gmt.keep(&mut [sid])).xpupil();
            let mut sensor = Builder::build(self.clone()).unwrap();
            sensor.calibrate(&mut src, threshold);
            let valid_lenslets: Vec<f32> = sensor.lenslet_mask().into();
            /*valid_lenslets.chunks(48).for_each(|row| {
                row.iter().for_each(|val| print!("{val:.2},"));
                println!("");
            });
            println!("");*/
            a.iter_mut()
                .zip(&valid_lenslets)
                .filter(|(_, v)| **v > 0.)
                .for_each(|(a, _)| {
                    *a += 1;
                });
            a
        });
        /*
        valid_lenslets.chunks(48).for_each(|row| {
            row.iter().for_each(|val| print!("{val}"));
            println!("");
        });*/
        valid_lenslets
            .iter_mut()
            .filter(|v| **v > 1)
            .for_each(|v| *v = 0);
        //dbg!(valid_lenslets.iter().cloned().sum::<i32>());
        let mut sensor = Builder::build(self.clone()).unwrap();
        let mut gmt = gmt_builder.clone().build()?;
        src.reset();
        src.through(&mut gmt);
        sensor.set_valid_lenslet(&valid_lenslets);
        sensor.set_reference_slopes(&mut src);
        Ok(sensor)
    }
}
impl SensorBuilder for SH24<Geometric> {
    type Sensor = ShackHartmann<Geometric>;
    fn build(self, gmt_builder: GMT, src_builder: SOURCE, threshold: f64) -> Result<Self::Sensor> {
        let mut src = self.guide_stars(Some(src_builder)).build()?;
        let n_side_lenslet = self.lenslet_array.0;
        let n = n_side_lenslet.pow(2) * self.n_sensor;
        let mut valid_lenslets: Vec<i32> = (1..=7).fold(vec![0i32; n as usize], |mut a, sid| {
            let mut gmt = gmt_builder.clone().build().unwrap();
            src.reset();
            src.through(gmt.keep(&mut [sid])).xpupil();
            let mut sensor = Builder::build(self.clone()).unwrap();
            sensor.calibrate(&mut src, threshold);
            let valid_lenslets: Vec<f32> = sensor.lenslet_mask().into();
            /*valid_lenslets.chunks(48).for_each(|row| {
                row.iter().for_each(|val| print!("{val:.2},"));
                println!("");
            });
            println!("");*/
            a.iter_mut()
                .zip(&valid_lenslets)
                .filter(|(_, v)| **v > 0.)
                .for_each(|(a, _)| {
                    *a += 1;
                });
            a
        });
        /*
        valid_lenslets.chunks(48).for_each(|row| {
            row.iter().for_each(|val| print!("{val}"));
            println!("");
        });*/
        valid_lenslets
            .iter_mut()
            .filter(|v| **v > 1)
            .for_each(|v| *v = 0);
        let mut sensor = Builder::build(self.clone()).unwrap();
        let mut gmt = gmt_builder.clone().build()?;
        src.reset();
        src.through(&mut gmt);
        sensor.set_valid_lenslet(&valid_lenslets);
        sensor.set_reference_slopes(&mut src);
        Ok(sensor)
    }
}
impl SensorBuilder for SH48<Geometric> {
    type Sensor = ShackHartmann<Geometric>;
    fn build(self, gmt_builder: GMT, src_builder: SOURCE, threshold: f64) -> Result<Self::Sensor> {
        let mut src = self.guide_stars(Some(src_builder)).build()?;
        let n_side_lenslet = self.lenslet_array.0;
        let n = n_side_lenslet.pow(2) * self.n_sensor;
        let mut valid_lenslets: Vec<i32> = (1..=7).fold(vec![0i32; n as usize], |mut a, sid| {
            let mut gmt = gmt_builder.clone().build().unwrap();
            src.reset();
            src.through(gmt.keep(&mut [sid])).xpupil();
            let mut sensor = Builder::build(self.clone()).unwrap();
            sensor.calibrate(&mut src, threshold);
            let valid_lenslets: Vec<f32> = sensor.lenslet_mask().into();
            /*valid_lenslets.chunks(48).for_each(|row| {
                row.iter().for_each(|val| print!("{val:.2},"));
                println!("");
            });
            println!("");*/
            a.iter_mut()
                .zip(&valid_lenslets)
                .filter(|(_, v)| **v > 0.)
                .for_each(|(a, _)| {
                    *a += 1;
                });
            a
        });
        /*
        valid_lenslets.chunks(48).for_each(|row| {
            row.iter().for_each(|val| print!("{val}"));
            println!("");
        });*/
        valid_lenslets
            .iter_mut()
            .filter(|v| **v > 1)
            .for_each(|v| *v = 0);
        let mut sensor = Builder::build(self.clone()).unwrap();
        let mut gmt = gmt_builder.clone().build()?;
        src.reset();
        src.through(&mut gmt);
        sensor.set_valid_lenslet(&valid_lenslets);
        sensor.set_reference_slopes(&mut src);
        Ok(sensor)
    }
}
/*
impl<T> SensorBuilder for T
where
    T: DerefMut<Target = ShackHartmannBuilder<Geometric>> + Default,
{
    type Sensor = ShackHartmann<Geometric>;
    fn build(self, gmt_builder: GMT, src_builder: SOURCE, threshold: f64) -> Result<Self::Sensor> {
        let mut src = self.guide_stars(Some(src_builder)).build()?;
        let n_side_lenslet = self.lenslet_array.0;
        let n = n_side_lenslet.pow(2) * self.n_sensor;
        let mut valid_lenslets: Vec<i32> = (1..=7).fold(vec![0i32; n as usize], |mut a, sid| {
            let mut gmt = gmt_builder.clone().build().unwrap();
            src.reset();
            src.through(gmt.keep(&mut [sid])).xpupil();
            let mut sensor = Builder::build(self.clone()).unwrap();
            sensor.calibrate(&mut src, threshold);
            let valid_lenslets: Vec<f32> = sensor.lenslet_mask().into();
            /*valid_lenslets.chunks(48).for_each(|row| {
                row.iter().for_each(|val| print!("{val:.2},"));
                println!("");
            });
            println!("");*/
            a.iter_mut()
                .zip(&valid_lenslets)
                .filter(|(_, v)| **v > 0.)
                .for_each(|(a, _)| {
                    *a += 1;
                });
            a
        });
        /*
        valid_lenslets.chunks(48).for_each(|row| {
            row.iter().for_each(|val| print!("{val}"));
            println!("");
        });*/
        valid_lenslets
            .iter_mut()
            .filter(|v| **v > 1)
            .for_each(|v| *v = 0);
        //dbg!(valid_lenslets.iter().cloned().sum::<i32>());
        let mut sensor = Builder::build(self.clone()).unwrap();
        let mut gmt = gmt_builder.clone().build()?;
        src.reset();
        src.through(&mut gmt);
        sensor.set_valid_lenslet(&valid_lenslets);
        sensor.set_reference_slopes(&mut src);
        Ok(sensor)
    }
}
*/
impl<S, B> OpticalModelBuilder<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone + SensorBuilder<Sensor = S>,
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
    pub fn sensor_builder(self, sensor_builder: B) -> Self {
        Self {
            sensor: Some(sensor_builder),
            ..self
        }
    }
    pub fn flux_threshold(self, flux_threshold: f64) -> Self {
        Self {
            flux_threshold,
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
            let sensor = <B as SensorBuilder>::build(
                sensor_builder.clone(),
                self.gmt.clone(),
                self.src.clone(),
                self.flux_threshold,
            )?;
            let src = sensor_builder.guide_stars(Some(self.src.clone())).build()?;
            let gmt = self.gmt.build()?;
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
pub struct OpticalModel<S = ShackHartmann<Geometric>, B = ShackHartmannBuilder<Geometric>>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
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
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone + SensorBuilder<Sensor = S>,
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
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
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

#[cfg(feature = "crseo")]
impl<S, B> Read<crseo::gmt::SegmentsDof, super::GmtState> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn read(&mut self, data: Arc<Data<crseo::gmt::SegmentsDof, super::GmtState>>) {
        if let Err(e) = &data.apply_to(&mut self.gmt) {
            crate::print_error("Failed applying GMT state", e);
        }
    }
}
impl<S, B> Read<Vec<f64>, super::M1rbm> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn read(&mut self, data: Arc<Data<Vec<f64>, super::M1rbm>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
impl<S, B> Read<Vec<f64>, super::M2rbm> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn read(&mut self, data: Arc<Data<Vec<f64>, super::M2rbm>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m2_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
#[cfg(feature = "fem")]
impl<S, B> Read<Vec<f64>, fem::fem_io::OSSM1Lcl> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
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
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
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
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::WfeRms>>> {
        Some(Arc::new(Data::new(self.src.wfe_rms())))
    }
}
impl<S, B> Write<Vec<f64>, super::SegmentWfeRms> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentWfeRms>>> {
        Some(Arc::new(Data::new(self.src.segment_wfe_rms())))
    }
}
impl<S, B> Write<Vec<f64>, super::SegmentPiston> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentPiston>>> {
        Some(Arc::new(Data::new(self.src.segment_piston())))
    }
}
impl<S, B> Write<Vec<f64>, super::SegmentGradients> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentGradients>>> {
        Some(Arc::new(Data::new(self.src.segment_gradients())))
    }
}
impl<S, B> Write<Vec<f64>, super::SegmentTipTilt> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentTipTilt>>> {
        Some(Arc::new(Data::new(self.src.segment_gradients())))
    }
}
impl<S, B> Write<Vec<f64>, super::PSSn> for OpticalModel<S, B>
where
    S: WavefrontSensor + Propagation,
    B: WavefrontSensorBuilder + Builder<Component = S> + Clone,
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
    for OpticalModel<ShackHartmann<Diffractive>, ShackHartmannBuilder<Diffractive>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.readout().process();
            let data: Vec<f64> = sensor.data().into();
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
    for OpticalModel<ShackHartmann<Geometric>, ShackHartmannBuilder<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = (mat * v) * 0.;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
impl Write<Vec<f64>, super::SensorData>
    for OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = (mat * v) * 0.;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
#[cfg(feature = "fsm")]
impl Write<Vec<f64>, crate::clients::fsm::TTFB>
    for OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
#[cfg(feature = "fsm")]
impl Write<Vec<f64>, crate::clients::fsm::TTFB>
    for OpticalModel<ShackHartmann<Geometric>, SH48<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
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
    for OpticalModel<ShackHartmann<Geometric>, SH48<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = (mat * v) * 0.;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
#[cfg(features = "fsm")]
impl Write<Vec<f64>, crate::clients::fsm::TTFB>
    for OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crseo::{Geometric, SH48};

    #[test]
    fn optical_model_calibration() {
        let mut optical_model = OpticalModel::builder()
            .sensor_builder(SH48::<Geometric>::new().n_sensor(1))
            .build()
            .unwrap();
        let valid_lenslets: Vec<f32> = optical_model.sensor.as_mut().unwrap().lenslet_mask().into();
        println!("Valid lenslets:");
        valid_lenslets.chunks(48).for_each(|row| {
            row.iter()
                .for_each(|val| print!("{}", if *val > 0. { 'x' } else { ' ' }));
            println!("");
        });
    }
}
