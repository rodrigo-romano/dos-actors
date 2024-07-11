use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use crseo::{
    wavefrontsensor::PhaseSensorBuilder, AtmosphereBuilder, Builder, GmtBuilder, SourceBuilder,
    WavefrontSensorBuilder,
};
use gmt_dos_clients_domeseeing::DomeSeeing;

use crate::OpticalModel;

use super::optical_model::OpticalModelError;

/// GMT optical model builder
///
/// ```no_run
/// use gmt_dos_clients_crseo::{OpticalModel};
/// use crseo::wavefrontsensor::PhaseSensor;
/// let optical_model_builder = OpticalModel::<PhaseSensor>::builder();
/// ```
#[derive(Debug, Default)]
pub struct OpticalModelBuilder<T = PhaseSensorBuilder> {
    gmt_builder: GmtBuilder,
    src_builder: SourceBuilder,
    atm_builder: Option<AtmosphereBuilder>,
    dome_seeing: Option<(PathBuf, usize)>,
    sampling_frequency: Option<f64>,
    piston: Option<Arc<Vec<f64>>>,
    sensor: Option<T>,
}
impl<T> OpticalModelBuilder<T> {
    /// Configures the GMT
    ///
    /// ```no_run
    /// # use gmt_dos_clients_crseo::OpticalModel;
    /// use crseo::{Gmt, FromBuilder, wavefrontsensor::PhaseSensor};
    /// let optical_model_builder = OpticalModel::<PhaseSensor>::builder().gmt(Gmt::builder());
    /// ```
    pub fn gmt(self, gmt_builder: GmtBuilder) -> Self {
        Self {
            gmt_builder,
            ..self
        }
    }
    /// Configures the light source
    ///
    /// ```no_run
    /// # use gmt_dos_clients_crseo::OpticalModel;
    /// use crseo::{Source, FromBuilder, wavefrontsensor::PhaseSensor};
    /// let optical_model_builder = OpticalModel::<PhaseSensor>::builder().source(Source::builder());
    /// ```
    pub fn source(self, src_builder: SourceBuilder) -> Self {
        Self {
            src_builder,
            ..self
        }
    }
    /// Adds a piston error of each segment to the wavefront in the exit pupil
    pub fn piston(self, piston: Vec<f64>) -> Self {
        Self {
            piston: Some(Arc::new(piston)),
            ..self
        }
    }
    /// Configures the atmospheric turbulence
    ///
    /// ```no_run
    /// # use gmt_dos_clients_crseo::OpticalModel;
    /// use crseo::{Atmosphere, FromBuilder, wavefrontsensor::PhaseSensor};
    /// let optical_model_builder = OpticalModel::<PhaseSensor>::builder().atmosphere(Atmosphere::builder());
    /// ```
    pub fn atmosphere(self, atm_builder: AtmosphereBuilder) -> Self {
        Self {
            atm_builder: Some(atm_builder),
            ..self
        }
    }
    /// Configures the dome seeing
    pub fn dome_seeing<P: AsRef<Path>>(mut self, path: P, upsampling: usize) -> Self {
        self.dome_seeing = Some((path.as_ref().to_owned(), upsampling));
        self
    }
    /// Sets the frequency in Hz to which the optical model is sampled
    ///
    /// ```no_run
    /// # use gmt_dos_clients_crseo::OpticalModel;
    /// use crseo::wavefrontsensor::PhaseSensor;
    /// let optical_model_builder = OpticalModel::<PhaseSensor>::builder().sampling_frequency(1000_f64);
    /// ```
    pub fn sampling_frequency(self, sampling_frequency: f64) -> Self {
        Self {
            sampling_frequency: Some(sampling_frequency),
            ..self
        }
    }
}

// impl OpticalModelBuilder {
//     /// Build the GMT optical model
//     ///
//     /// ```no_run
//     /// # use gmt_dos_clients_crseo::OpticalModel;
//     /// let optical_model_builder = OpticalModel::builder().build()?;
//     /// # Ok::<(), Box<dyn std::error::Error>>(())
//     /// ```
//     pub fn build(self) -> Result<OpticalModel, OpticalModelError> {
//         let gmt = self.gmt_builder.build()?;
//         let src = Rc::new(RefCell::new(self.src_builder.build()?));
//         let atm = if let Some(atm_builder) = self.atm_builder {
//             Some(atm_builder.build()?)
//         } else {
//             None
//         };
//         let dome_seeing = if let Some((path, upsampling)) = self.dome_seeing {
//             Some(DomeSeeing::new(path.to_str().unwrap(), upsampling, None)?)
//         } else {
//             None
//         };
//         Ok(OpticalModel {
//             gmt,
//             src,
//             atm,
//             dome_seeing,
//             tau: self.sampling_frequency.map_or_else(|| 0f64, |x| x.recip()),
//             piston: self.piston,
//             sensor: None,
//         })
//     }
// }

impl<T: Builder + WavefrontSensorBuilder> OpticalModelBuilder<T> {
    /// Sets the wavefront sensor
    pub fn sensor(mut self, sensors: T) -> Self {
        self.sensor = Some(sensors);
        self
    }
    #[deprecated(since  = "4.0.1", note="use `sensor` instead")]
    pub fn pyramid(mut self, sensors: T) -> Self {
        self.sensor = Some(sensors);
        self
    }
    /// Build the GMT optical model
    ///
    /// ```no_run
    /// # use gmt_dos_clients_crseo::OpticalModel;
    /// use crseo::wavefrontsensor::PhaseSensor;
    /// let optical_model_builder = OpticalModel::<PhaseSensor>::builder().build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn build(self) -> Result<OpticalModel<T::Component>, OpticalModelError> {
        let gmt = self.gmt_builder.build()?;
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
        let (sensor, src_builder) = if let Some(sensor) = self.sensor {
            let src_builder =
                <T as WavefrontSensorBuilder>::guide_stars(&sensor, Some(self.src_builder));
            (Some(sensor.build()?), src_builder)
        } else {
            (None, self.src_builder)
        };
        Ok(OpticalModel {
            gmt,
            src: Rc::new(RefCell::new(src_builder.build()?)),
            atm,
            dome_seeing,
            tau: self.sampling_frequency.map_or_else(|| 0f64, |x| x.recip()),
            piston: self.piston,
            sensor,
        })
    }
}
