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
pub enum SegmentTipTilt {}
/// Source PSSn
pub enum PSSn {}
/// Sensor data
pub enum SensorData {}
/// M1 rigid body motions
pub enum M1rbm {}
/// M2 rigid body motions
pub enum M2rbm {}
/// GMT M1 &M1 state
pub enum GmtState {}
