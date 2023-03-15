/*!
#  CEO Optical Model

This module is a high-level interface to [crseo] and [crseo] is a Rust wrapper around CEO.
CEO is a CUDA-based optical propagation model for the GMT.

Follow the instructions [here](https://github.com/rconan/crseo) to install and to setup CEO.

A default optical model consists in the GMT and an on-axis source
```
use dos_actors::prelude::*;
use gmt_dos_clients_crseo::OpticalModel;
let optical_model = OpticalModel::builder().build().expect("Failed to build CEO optical model");
```
 */

use gmt_dos_clients::interface::{Size, UniqueIdentifier, UID};
use gmt_dos_clients_io::{
    gmt_m1::{M1ModeShapes, M1RigidBodyMotions},
    gmt_m2::M2RigidBodyMotions,
};

pub(crate) mod optical_model;
pub use optical_model::{
    OpticalModel, OpticalModelBuilder, OpticalModelOptions, PSSnOptions, ShackHartmannOptions,
};
pub(crate) mod shackhartmann;

mod sensor;
pub use sensor::SensorBuilder;

mod error;
pub use error::{CeoError, Result};

/// Source wavefront error RMS `[m]`
#[derive(UID)]
pub enum WfeRms {}
impl Size<WfeRms> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize
    }
}
/// Wavefront in the exit pupil \[m\]
#[derive(UID)]
#[uid(data = "Vec<f32>")]
pub enum Wavefront {}
impl Size<Wavefront> for OpticalModel {
    fn len(&self) -> usize {
        let n = self.src.pupil_sampling as usize;
        self.src.size as usize * n * n
    }
}
/// Source wavefront gradient pupil average `2x[rd]`
#[derive(UID)]
pub enum TipTilt {}
impl Size<TipTilt> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize * 2
    }
}
/// Source segment wavefront piston and standard deviation `([m],[m])x7`
#[derive(UID)]
pub enum SegmentWfe {}
impl Size<SegmentWfe> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize * 7 * 2
    }
}
/// Source segment wavefront error RMS `7x[m]`
#[derive(UID)]
pub enum SegmentWfeRms {}
impl Size<SegmentWfeRms> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize * 7
    }
}
/// Source segment piston `7x[m]`
#[derive(UID)]
pub enum SegmentPiston {}
impl Size<SegmentPiston> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize * 7
    }
}
/// Source segment tip-tilt `[7x[rd],7x[rd]]`
#[derive(UID)]
pub enum SegmentGradients {}
impl Size<SegmentGradients> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize * 14
    }
}
#[derive(UID)]
pub enum SegmentTipTilt {}
impl Size<SegmentTipTilt> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize * 14
    }
}
/// Source PSSn
#[derive(UID)]
pub enum PSSn {}
impl Size<PSSn> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize
    }
}
/// Source PSSn and FWHM
#[derive(UID)]
pub enum PSSnFwhm {}
impl Size<PSSnFwhm> for OpticalModel {
    fn len(&self) -> usize {
        self.src.size as usize * 2
    }
}
/// Read-out and return sensor data
#[derive(UID)]
pub enum SensorData {}
/// Detector frame
#[derive(UID)]
#[uid(data = "Vec<f32>")]
pub enum DetectorFrame {}
/// M2 mode coeffcients
#[derive(UID)]
pub enum M2modes {}
/// M2 Rx and Ry rigid body motions
#[derive(UID)]
pub enum M2rxy {}
#[cfg(feature = "crseo")]
/// GMT M1 & M2 state
#[derive(UID)]
#[uid(data = "crseo::gmt::SegmentsDof")]
pub enum GmtState {}
pub enum PointingError {}
impl UniqueIdentifier for PointingError {
    type DataType = (f64, f64);
}

#[cfg(feature = "fem")]
impl<S> dos_actors::io::Write<M1modes> for fem::dos::DiscreteModalSolver<S>
where
    S: fem::dos::Solver + Default,
{
    fn write(&mut self) -> Option<std::sync::Arc<dos_actors::io::Data<M1modes>>> {
        let mut data: std::sync::Arc<dos_actors::io::Data<fem::dos::M1SegmentsAxialD>> =
            self.write()?;
        let inner = std::sync::Arc::get_mut(&mut data)?;
        Some(std::sync::Arc::new(inner.into()))
    }
}
