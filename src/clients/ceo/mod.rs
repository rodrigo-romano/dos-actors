/*!
#  CEO Optical Model

This module is a high-level interface to [crseo] and [crseo] is a Rust wrapper around CEO.
CEO is a CUDA-based optical propagation model for the GMT.

The [ceo] client is enabled with the `ceo` feature.

A default optical model consists in the GMT and an on-axis source
```
use crate::prelude::*;
use crate::clients::ceo;
let optical_model = OpticalModel::builder().build()?;
# Ok::<(), dos_actors::clients::ceo::CeoError>(())
```
 */

use uid::UniqueIdentifier;
use uid_derive::UID;

pub(crate) mod optical_model;
pub use optical_model::{
    OpticalModel, OpticalModelBuilder, OpticalModelOptions, PSSnOptions, ShackHartmannOptions,
};
pub(crate) mod shackhartmann;

/// Source wavefront error RMS `[m]`
#[derive(UID)]
pub enum WfeRms {}
/// Source wavefront gradient pupil average `2x[rd]`
#[derive(UID)]
pub enum TipTilt {}
/// Source segment wavefront error RMS `7x[m]`
#[derive(UID)]
pub enum SegmentWfeRms {}
/// Source segment piston `7x[m]`
#[derive(UID)]
pub enum SegmentPiston {}
/// Source segment tip-tilt `[7x[rd],7x[rd]]`
#[derive(UID)]
pub enum SegmentGradients {}
#[derive(UID)]
pub enum SegmentTipTilt {}
/// Source PSSn
#[derive(UID)]
pub enum PSSn {}
/// Sensor data
#[derive(UID)]
pub enum SensorData {}
/// Detector frame
pub enum DetectorFrame {}
impl UniqueIdentifier for DetectorFrame {
    type Data = Vec<f32>;
}
/// M1 rigid body motions
#[derive(UID)]
pub enum M1rbm {}
/// M1 mode coeffcients
#[derive(UID)]
pub enum M1modes {}
/// M2 rigid body motions
#[derive(UID)]
pub enum M2rbm {}
#[cfg(feature = "crseo")]
/// GMT M1 &M1 state
pub enum GmtState {}
#[cfg(feature = "crseo")]
impl UniqueIdentifier for GmtState {
    type Data = crseo::gmt::SegmentsDof;
}
