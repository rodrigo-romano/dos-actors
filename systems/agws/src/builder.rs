pub mod shack_hartmann;

use std::path::Path;

use gmt_dos_actors::system::{Sys, SystemError};
use gmt_dos_clients_crseo::{
    calibration::Reconstructor,
    crseo::{
        builders::{AtmosphereBuilder, AtmosphereBuilderError},
        imaging::{Detector, LensletArray},
        FromBuilder,
    },
    sensors::{builders::CameraBuilder, Camera},
    OpticalModel, OpticalModelBuilder, OpticalModelError,
};
use shack_hartmann::{AgwsGuideStar, ShackHartmannBuilder, ShackHartmannBuilderError};

use crate::{
    agws::{sh24::Sh24, sh48::Sh48},
    kernels::{Kernel, KernelError},
    Agws,
};

#[derive(Debug, thiserror::Error)]
pub enum AgwsBuilderError {
    #[error("failed to build AGWS optical model")]
    OpticalModel(#[from] OpticalModelError),
    #[error("AGWS kernel error")]
    AgwsKernel(#[from] KernelError),
    #[error("AGWS system error")]
    AgwsSystem(#[from] SystemError),
    #[error("failed load atmospheric turbulence model")]
    Atmosphere(#[from] AtmosphereBuilderError),
    #[error("ShackHartmann builder error")]
    ShackHartmann(#[from] ShackHartmannBuilderError),
}

pub struct AgwsShackHartmann;
impl AgwsShackHartmann {
    /// 48x48 Shack-Hartmann wavefront sensor
    ///
    /// The pixel scale is 0.4" and the lenslet field-of-view is 3.2"
    pub fn sh48<const I: usize>() -> CameraBuilder<I> {
        Camera::<I>::builder()
            .n_sensor(3)
            .lenslet_array(LensletArray::default().n_side_lenslet(48).n_px_lenslet(18))
            .detector(Detector::default().n_px_imagelet(24).n_px_framelet(8))
            .lenslet_flux(0.75)
    }
    /// 24x24 Shack-Hartmann wavefront sensor
    ///
    /// The pixel scale is 0.4" and the lenslet field-of-view is 4.8"
    pub fn sh24<const I: usize>() -> CameraBuilder<I> {
        Camera::<I>::builder()
            .lenslet_array(LensletArray::default().n_side_lenslet(24).n_px_lenslet(36))
            .detector(Detector::default().n_px_imagelet(72).n_px_framelet(12))
            .lenslet_flux(0.75)
    }
}

/// [Agws] builder
#[derive(Debug, Clone)]
pub struct AgwsBuilder<const SH48_I: usize = 1, const SH24_I: usize = 1> {
    sh48: OpticalModelBuilder<CameraBuilder<SH48_I>>,
    // sh24: OpticalModelBuilder<CameraBuilder<SH24_I>>,
    // sh24_recon: Option<Reconstructor>,
    sh24: ShackHartmannBuilder<SH24_I>,
    atm: Option<(AtmosphereBuilder, f64)>,
}

impl<const SH48_I: usize, const SH24_I: usize> Default for AgwsBuilder<SH48_I, SH24_I> {
    fn default() -> Self {
        let sh48 = OpticalModel::<Camera<SH48_I>>::builder()
            .sensor(AgwsShackHartmann::sh48())
            .source(AgwsGuideStar::sh48());
        Self {
            sh48,
            sh24: ShackHartmannBuilder::<SH24_I>::sh24(),
            // sh24_recon: None,
            atm: None,
        }
    }
}

impl<const SH48_I: usize, const SH24_I: usize> AgwsBuilder<SH48_I, SH24_I> {
    /// Create a new builder instance
    pub fn new() -> Self {
        Default::default()
    }
    /// Loads the atmospheric turbulence model sampled at the given frequency in Hz
    pub fn load_atmosphere(
        mut self,
        path: impl AsRef<Path>,
        sampling_frequency: f64,
    ) -> Result<Self, AgwsBuilderError> {
        self.atm = Some((AtmosphereBuilder::load(path)?, sampling_frequency));
        Ok(self)
    }
    /// Returns a clone of the AGWS SH48 builder
    pub fn sh48(&self) -> OpticalModelBuilder<CameraBuilder<SH48_I>> {
        self.sh48.clone()
    }
    /// Sets the AGWS SH24 builder
    pub fn sh24(mut self, sh24: ShackHartmannBuilder<SH24_I>) -> Self {
        self.sh24 = sh24;
        self
    }
    /// Sets the path to SH24 calibration file
    pub fn sh24_calibration(mut self, sh24_recon: Reconstructor) -> Self {
        self.sh24 = self.sh24.reconstructor(sh24_recon);
        self
    }
    /// Build an [Agws] [system](gmt_dos_actors::system::Sys) instance
    pub fn build(mut self) -> Result<Sys<Agws<SH48_I, SH24_I>>, AgwsBuilderError> {
        if let Some((ref atm, sampling_frequency)) = self.atm {
            self.sh48 = self
                .sh48
                .atmosphere(atm.clone())
                .sampling_frequency(sampling_frequency);
        }
        let sh48 = self.sh48.build()?;
        log::info!("SH48:\n{}", sh48);
        // let calib_sh24 = self.sh24.clone().source(AgwsGuideStar::sh24().fwhm(12.));
        // let sh24_kernel = Kernel::<Sh24<SH24_I>>::new(&calib_sh24, sh24_recon)?;
        let mut sh24 = OpticalModelBuilder::from(self.sh24.clone());
        if let Some((atm, sampling_frequency)) = self.atm {
            sh24 = sh24
                .atmosphere(atm.clone())
                .sampling_frequency(sampling_frequency);
        }
        let sh24 = sh24.build()?;
        log::info!("SH24:\n{}", sh24);
        Ok(Sys::new(Agws {
            sh48: Sh48(sh48).into(),
            sh24: Sh24(sh24).into(),
            sh24_kernel: Kernel::<Sh24<SH24_I>>::try_from(&self.sh24)?.into(),
        })
        .build()?)
    }
}
