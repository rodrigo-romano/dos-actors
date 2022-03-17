mod optical_model;
pub use optical_model::{OpticalModel, OpticalModelBuilder, SensorBuilder};

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
