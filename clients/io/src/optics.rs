use interface::UID;

/// Source wavefront error RMS `[m]`
#[derive(UID)]
pub enum WfeRms {}

/// Wavefront in the exit pupil \[m\]
#[derive(UID)]
#[uid(data = Vec<f32>, port = 55_001)]
pub enum Wavefront {}

/// Source wavefront gradient pupil average `2x[rd]`
#[derive(UID)]
#[uid(port = 55_002)]
pub enum TipTilt {}

/// Source segment wavefront piston and standard deviation `([m],[m])x7`
#[derive(UID)]
#[uid(port = 55_003)]
pub enum SegmentWfe {}

/// Source segment wavefront error RMS `7x[m]`
#[derive(UID)]
#[uid(port = 55_004)]
pub enum SegmentWfeRms {}

/// Source segment piston `7x[m]`
#[derive(UID)]
#[uid(port = 55_005)]
pub enum SegmentPiston {}

/// Source segment tip-tilt `[7x[rd],7x[rd]]`
#[derive(UID)]
#[uid(port = 55_006)]
pub enum SegmentTipTilt {}

/// Read-out and return sensor data
#[derive(UID)]
#[uid(port = 55_007)]
pub enum SensorData {}

/// Detector frame
#[derive(UID)]
#[uid(data = Vec<f32>, port = 55_008)]
pub enum DetectorFrame {}

/// M2 mode coefficients
#[derive(UID)]
#[uid(port = 55_009)]
pub enum M2modes {}

/// M2 Rx and Ry rigid body motions
#[derive(UID)]
#[uid(port = 55_010)]
pub enum M2rxy {}
