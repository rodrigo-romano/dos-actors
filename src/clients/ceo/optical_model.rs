use crate::{
    io::{Data, Read, Write},
    Update,
};
use crseo::{
    pssn::TelescopeError, Atmosphere, Builder, Gmt, PSSn, Source, WavefrontSensor,
    WavefrontSensorBuilder, ATMOSPHERE, GMT, PSSN, SOURCE,
};
use nalgebra as na;
use std::{ops::DerefMut, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum CeoError {
    #[error("CEO building failed")]
    CEO(#[from] crseo::CrseoError),
}
pub type Result<T> = std::result::Result<T, CeoError>;

/// GMT optical model builder
pub struct OpticalModelBuilder {
    gmt: GMT,
    src: SOURCE,
    atm: Option<ATMOSPHERE>,
    pssn: Option<PSSN<TelescopeError>>,
    flux_threshold: f64,
    tau: f64,
}
impl Default for OpticalModelBuilder {
    fn default() -> Self {
        Self {
            gmt: GMT::default(),
            src: SOURCE::default(),
            atm: None,
            pssn: None,
            flux_threshold: 0.1,
            tau: 0f64,
        }
    }
}

pub trait SensorBuilder: WavefrontSensorBuilder + Builder + Clone {
    fn build(
        self,
        gmt_builder: GMT,
        src_builder: SOURCE,
        threshold: f64,
    ) -> Result<Box<dyn WavefrontSensor>>;
}

impl OpticalModelBuilder {
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
    /*
    /// Sets the `sensor` builder
    fn sensor_builder(self, sensor_builder: B) -> Self {
        Self {
            sensor: Some(sensor_builder),
            ..self
        }
    }*/
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
    pub fn pssn(self, pssn: PSSN<TelescopeError>) -> Self {
        Self {
            pssn: Some(pssn),
            ..self
        }
    }
    /// Sets the sampling period
    pub fn sampling_period(self, tau: f64) -> Self {
        Self { tau, ..self }
    }
    /// Builds a new GMT optical model
    ///
    /// If there is `Some` sensor, it is initialized.
    pub fn build_with(self, sensor_builder: impl SensorBuilder) -> Result<OpticalModel> {
        let sensor = SensorBuilder::build(
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
            frame: None,
            tau: self.tau,
        })
    }
    pub fn build(self) -> Result<OpticalModel> {
        /*
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
                        frame: None,
                        tau: self.tau,
                    })
                } else {
        */
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
            frame: None,
            tau: self.tau,
        })
        //      }
    }
}
pub enum SensorFn {
    None,
    Fn(Box<dyn Fn(Vec<f64>) -> Vec<f64> + Send>),
    Matrix(na::DMatrix<f64>),
}
/// GMT Optical Model
pub struct OpticalModel {
    pub gmt: Gmt,
    pub src: Source,
    pub sensor: Option<Box<dyn WavefrontSensor>>,
    pub atm: Option<Atmosphere>,
    pub pssn: Option<PSSn<TelescopeError>>,
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
        if let Some(sensor) = &mut self.sensor {
            //self.src.through(sensor);
            sensor.deref_mut().propagate(&mut self.src);
        }
        if let Some(pssn) = &mut self.pssn {
            self.src.through(pssn);
        }
    }
}

impl Read<crate::prelude::Void, crate::prelude::Tick> for OpticalModel {
    fn read(&mut self, _: Arc<Data<crate::prelude::Void, crate::prelude::Tick>>) {}
}

#[cfg(feature = "crseo")]
impl Read<crseo::gmt::SegmentsDof, super::GmtState> for OpticalModel {
    fn read(&mut self, data: Arc<Data<crseo::gmt::SegmentsDof, super::GmtState>>) {
        if let Err(e) = &data.apply_to(&mut self.gmt) {
            crate::print_error("Failed applying GMT state", e);
        }
    }
}
impl Read<Vec<f64>, super::M1rbm> for OpticalModel {
    fn read(&mut self, data: Arc<Data<Vec<f64>, super::M1rbm>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
impl Read<Vec<f64>, super::M1modes> for OpticalModel {
    fn read(&mut self, data: Arc<Data<Vec<f64>, super::M1modes>>) {
        self.gmt.m1_modes(&data);
    }
}
impl Read<Vec<f64>, super::M2rbm> for OpticalModel {
    fn read(&mut self, data: Arc<Data<Vec<f64>, super::M2rbm>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m2_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
#[cfg(feature = "fem")]
impl Read<Vec<f64>, fem::fem_io::OSSM1Lcl> for OpticalModel {
    fn read(&mut self, data: Arc<Data<Vec<f64>, fem::fem_io::OSSM1Lcl>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
#[cfg(feature = "fem")]
impl Read<Vec<f64>, fem::fem_io::MCM2Lcl6D> for OpticalModel {
    fn read(&mut self, data: Arc<Data<Vec<f64>, fem::fem_io::MCM2Lcl6D>>) {
        data.chunks(6).enumerate().for_each(|(sid0, v)| {
            self.gmt
                .m2_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
        });
    }
}
impl Write<Vec<f64>, super::WfeRms> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::WfeRms>>> {
        Some(Arc::new(Data::new(self.src.wfe_rms())))
    }
}
impl Write<Vec<f64>, super::TipTilt> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::TipTilt>>> {
        Some(Arc::new(Data::new(self.src.gradients())))
    }
}
impl Write<Vec<f64>, super::SegmentWfeRms> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentWfeRms>>> {
        Some(Arc::new(Data::new(self.src.segment_wfe_rms())))
    }
}
impl Write<Vec<f64>, super::SegmentPiston> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentPiston>>> {
        Some(Arc::new(Data::new(self.src.segment_piston())))
    }
}
impl Write<Vec<f64>, super::SegmentGradients> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentGradients>>> {
        Some(Arc::new(Data::new(self.src.segment_gradients())))
    }
}
impl Write<Vec<f64>, super::SegmentTipTilt> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SegmentTipTilt>>> {
        Some(Arc::new(Data::new(self.src.segment_gradients())))
    }
}
impl Write<Vec<f64>, super::PSSn> for OpticalModel {
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
