use crate::{
    io::{Data, Read, Write},
    Update,
};
use crseo::{
    pssn::TelescopeError, Atmosphere, Builder, Diffractive, Geometric, Gmt, PSSn, Propagation,
    ShackHartmann, Source, WavefrontSensor, WavefrontSensorBuilder, ATMOSPHERE, GMT, PSSN,
    SHACKHARTMANN, SOURCE,
};
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum CeoError {
    #[error("CEO building failed")]
    CEO(#[from] crseo::CrseoError),
}
pub type Result<T> = std::result::Result<T, CeoError>;

/// GMT optical model builder
pub struct OpticalModelBuilder<
    Sensor = ShackHartmann<Geometric>,
    SensorBuilder = SHACKHARTMANN<Geometric>,
> where
    Sensor: WavefrontSensor + Propagation,
    SensorBuilder: WavefrontSensorBuilder + Builder<Component = Sensor>,
{
    pub gmt: GMT,
    pub src: SOURCE,
    pub atm: Option<ATMOSPHERE>,
    pub sensor: Option<SensorBuilder>,
    pub pssn: Option<PSSN<TelescopeError>>,
}
impl<Sensor, SensorBuilder> OpticalModelBuilder<Sensor, SensorBuilder>
where
    Sensor: WavefrontSensor + Propagation,
    SensorBuilder: WavefrontSensorBuilder + Builder<Component = Sensor>,
{
    /// Creates a new GMT optical model
    ///
    /// Creates a default model based on the default parameters for [GMT] and [SOURCE]
    pub fn new() -> Self {
        Self {
            gmt: GMT::default(),
            src: SOURCE::default(),
            atm: None,
            sensor: None,
            pssn: None,
        }
    }
    /// Sets the GMT model
    pub fn gmt(self, gmt: GMT) -> Self {
        Self { gmt, ..self }
    }
    /// Sets the `Source` model
    pub fn source(self, src: SOURCE) -> Self {
        Self { src, ..self }
    }
    /// Sets the [atmosphere](ATMOSPHERE) template
    pub fn atmosphere(self, atm: ATMOSPHERE) -> Self {
        Self {
            atm: Some(atm),
            ..self
        }
    }
    /// Builds a new GMT optical model
    pub fn build(self) -> Result<OpticalModel<Sensor>> {
        Ok(OpticalModel {
            gmt: self.gmt.build()?,
            src: self.src.clone().build()?,
            sensor: self.sensor.map(|sensor| sensor.build().unwrap()),
            atm: match self.atm {
                Some(atm) => Some(atm.build()?),
                None => None,
            },
            pssn: match self.pssn {
                Some(pssn) => Some(pssn.source(&(self.src.build()?)).build()?),
                None => None,
            },
        })
    }
}
/// GMT Optical Model
pub struct OpticalModel<Sensor = ShackHartmann<Geometric>>
where
    Sensor: Propagation,
{
    pub gmt: Gmt,
    pub src: Source,
    pub sensor: Option<Sensor>,
    pub atm: Option<Atmosphere>,
    pub pssn: Option<PSSn<TelescopeError>>,
}

impl OpticalModel<ShackHartmann<Geometric>> {
    pub fn builder() -> OpticalModelBuilder<ShackHartmann<Geometric>, SHACKHARTMANN<Geometric>> {
        OpticalModelBuilder::new()
    }
}

impl<Sensor: Propagation> Update for OpticalModel<Sensor> {
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
impl<Sensor: Propagation> Read<Vec<f64>, fem::fem_io::OSSM1Lcl> for OpticalModel<Sensor> {
    fn read(&mut self, data: Arc<Data<Vec<f64>, fem::fem_io::OSSM1Lcl>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
#[cfg(feature = "fem")]
impl<Sensor: Propagation> Read<Vec<f64>, fem::fem_io::MCM2Lcl6D> for OpticalModel<Sensor> {
    fn read(&mut self, data: Arc<Data<Vec<f64>, fem::fem_io::MCM2Lcl6D>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m2_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
impl<Sensor: Propagation> Write<Vec<f64>, super::WfeRms> for OpticalModel<Sensor> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::WfeRms>>> {
        Some(Arc::new(Data::new(self.src.wfe_rms())))
    }
}
impl<Sensor: Propagation> Write<Vec<f64>, super::SegmentWfeRms> for OpticalModel<Sensor> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentWfeRms>>> {
        Some(Arc::new(Data::new(self.src.segment_wfe_rms())))
    }
}
impl<Sensor: Propagation> Write<Vec<f64>, super::SegmentPiston> for OpticalModel<Sensor> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentPiston>>> {
        Some(Arc::new(Data::new(self.src.segment_piston())))
    }
}
impl<Sensor: Propagation> Write<Vec<f64>, super::SegmentGradients> for OpticalModel<Sensor> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentGradients>>> {
        Some(Arc::new(Data::new(self.src.segment_gradients())))
    }
}
impl<Sensor: Propagation> Write<Vec<f64>, super::PSSn> for OpticalModel<Sensor> {
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
impl Write<Vec<f64>, super::SensorData> for OpticalModel<ShackHartmann<Diffractive>> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.readout().process();
            let data: Vec<f32> = sensor.get_data().into();
            sensor.reset();
            Some(Arc::new(Data::new(
                data.into_iter().map(|x| x as f64).collect(),
            )))
        } else {
            None
        }
    }
}
impl Write<Vec<f64>, super::SensorData> for OpticalModel<ShackHartmann<Geometric>> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f32> = sensor.get_data().into();
            sensor.reset();
            Some(Arc::new(Data::new(
                data.into_iter().map(|x| x as f64).collect(),
            )))
        } else {
            None
        }
    }
}
