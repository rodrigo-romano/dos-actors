use gmt_dos_actors::system::Sys;
use gmt_dos_clients_crseo::{
    crseo::{
        imaging::{Detector, LensletArray},
        FromBuilder, Source,
    },
    sensors::{builders::CameraBuilder, Camera},
    OpticalModel, OpticalModelBuilder, OpticalModelError,
};
use skyangle::Conversion;

use crate::{
    agws::{sh24::Sh24, sh48::Sh48},
    Agws,
};

#[derive(Debug, thiserror::Error)]
pub enum AgwsBuilderError {
    #[error("failed to build AGWS optical model")]
    OpticalModel(#[from] OpticalModelError),
}

#[derive(Debug, Clone)]
pub struct AgwsBuilder<const SH48_I: usize = 1, const SH24_I: usize = 1> {
    sh48: OpticalModelBuilder<CameraBuilder<SH48_I>>,
    sh24: OpticalModelBuilder<CameraBuilder<SH24_I>>,
}

impl<const SH48_I: usize, const SH24_I: usize> Default for AgwsBuilder<SH48_I, SH24_I> {
    fn default() -> Self {
        let sh48 = OpticalModel::<Camera<SH48_I>>::builder()
            .sensor(
                Camera::<SH48_I>::builder()
                    .n_sensor(3)
                    .lenslet_array(LensletArray::default().n_side_lenslet(48).n_px_lenslet(24))
                    .detector(Detector::default().n_px_imagelet(8))
                    .lenslet_flux(0.75),
            )
            .source(Source::builder().size(3).zenith_azimuth(
                vec![6f32.from_arcmin(), 5f32.from_arcmin(), 7f32.from_arcmin()],
                vec![15f32.to_radians(), 135f32.to_radians(), 255f32.to_radians()],
            ));
        let sh24 = OpticalModel::<Camera<SH24_I>>::builder()
            .sensor(
                Camera::<SH24_I>::builder()
                    .lenslet_array(LensletArray::default().n_side_lenslet(24).n_px_lenslet(24))
                    .detector(Detector::default().n_px_imagelet(16).n_px_framelet(8)),
            )
            .source(
                Source::builder()
                    .zenith_azimuth(vec![4f32.from_arcmin()], vec![180f32.to_radians()]),
            );
        Self { sh48, sh24 }
    }
}

impl<const SH48_I: usize, const SH24_I: usize> AgwsBuilder<SH48_I, SH24_I> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn build(self) -> anyhow::Result<Sys<Agws<SH48_I, SH24_I>>> {
        let sh48 = self.sh48.build()?;
        log::info!("SH48:\n{}", sh48);
        let sh24 = self.sh24.build()?;
        log::info!("SH24:\n{}", sh24);
        Ok(Sys::new(Agws {
            sh48: Sh48(sh48).into(),
            sh24: Sh24(sh24).into(),
        })
        .build()?)
    }
}
