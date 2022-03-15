//use crseo::{Builder, Diffractive, Geometric, ShackHartmann, WavefrontSensorBuilder, SH24, SH48};

mod optical_model;
pub use optical_model::{OpticalModel, OpticalModelBuilder};

/// Source wavefront error RMS `[m]`
pub enum WfeRms {}
/// Source segment wavefront error RMS `7x[m]`
pub enum SegmentWfeRms {}
/// Source segment piston `7x[m]`
pub enum SegmentPiston {}
/// Source segment tip-tilt `[7x[rd],7x[rd]]`
pub enum SegmentGradients {}
/// Source PSSn
pub enum PSSn {}
/// Sensor data
pub enum SensorData {}

/*
impl OpticalModel<ShackHartmann<Geometric>> {
    pub fn builder() -> OpticalModelBuilder<ShackHartmann<Geometric>, SH48<Geometric>> {
        OpticalModelBuilder {
            gmt: Default::default(),
            src: SH48::<Geometric>::new().guide_stars(None),
            atm: None,
            sensor: Some(SH48::new()),
            pssn: None,
        }
    }
}

impl OpticalModel<ShackHartmann<Diffractive>> {
    pub fn builder() -> OpticalModelBuilder<ShackHartmann<Diffractive>, SH48<Diffractive>> {
        OpticalModelBuilder {
            gmt: Default::default(),
            src: SH48::<Diffractive>::new().guide_stars(None),
            atm: None,
            sensor: Some(SH48::new()),
            pssn: None,
        }
    }
}

impl OpticalModel<ShackHartmann<Geometric>> {
    pub fn builder() -> OpticalModelBuilder<ShackHartmann<Geometric>, SH24<Geometric>> {
        OpticalModelBuilder {
            gmt: Default::default(),
            src: SH24::<Geometric>::new().guide_stars(None),
            atm: None,
            sensor: Some(SH24::new()),
            pssn: None,
        }
    }
}

impl OpticalModel<ShackHartmann<Diffractive>> {
    pub fn builder() -> OpticalModelBuilder<ShackHartmann<Diffractive>, SH24<Diffractive>> {
        OpticalModelBuilder {
            gmt: Default::default(),
            src: SH24::<Diffractive>::new().guide_stars(None),
            atm: None,
            sensor: Some(SH24::new()),
            pssn: None,
        }
    }
}
*/
