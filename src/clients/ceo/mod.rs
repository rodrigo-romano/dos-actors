/*!
#  CEO Optical Model

This module is a high-level interface to [crseo] and [crseo] is a Rust wrapper around CEO.
CEO is a CUDA-based optical propagation model for the GMT.

A default optical model consists in the GMT and an on-axis source
```
use crate::prelude::*;
use crate::clients::ceo;
let optical_model = OpticalModel::builder().build()?;
# Ok::<(), dos_actors::clients::ceo::CeoError>(())
```
 */

pub(crate) mod optical_model;
pub use optical_model::{
    OpticalModel, OpticalModelBuilder, OpticalModelOptions, PSSnOptions, ShackHartmannOptions,
};
pub(crate) mod shackhartmann;

/// Source wavefront error RMS `[m]`
pub enum WfeRms {}
/// Source wavefront gradient pupil average `2x[rd]`
pub enum TipTilt {}
/// Source segment wavefront error RMS `7x[m]`
pub enum SegmentWfeRms {}
/// Source segment piston `7x[m]`
pub enum SegmentPiston {}
/// Source segment tip-tilt `[7x[rd],7x[rd]]`
pub enum SegmentGradients {}
pub enum SegmentTipTilt {}
/// Source PSSn
pub enum PSSn {}
/// Sensor data
pub enum SensorData {}
/// Detector frame
pub enum DetectorFrame {}
/// M1 rigid body motions
pub enum M1rbm {}
/// M1 mode coeffcients
pub enum M1modes {}
/// M2 rigid body motions
pub enum M2rbm {}
/// GMT M1 &M1 state
pub enum GmtState {}
