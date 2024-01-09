use interface::{UniqueIdentifier, UID};

/// Source wavefront error RMS `[m]`
#[derive(UID)]
#[uid(port = 55_011)]
pub enum WfeRms<const E: i32 = 0> {}

/// Wavefront within the exit pupil \[m\]
#[derive(UID)]
#[uid(data = Vec<f64>, port = 55_001)]
pub enum MaskedWavefront {}
/// Wavefront in the exit pupil \[m\]
#[derive(UID)]
#[uid(data = Vec<f64>, port = 55_001)]
pub enum Wavefront {}

/// Source wavefront gradient pupil average `2x[rd]`
#[derive(UID)]
#[uid(port = 55_002)]
pub enum TipTilt {}

/// Source segment wavefront piston and standard deviation `([m],[m])x7`
pub enum SegmentWfe<const E: i32 = 0> {}
impl<const E: i32> UniqueIdentifier for SegmentWfe<E> {
    type DataType = Vec<(f64, f64)>;
    const PORT: u32 = 55_003;
}
pub enum SegmentDWfe<const E: i32 = 0> {}
impl<const E: i32> UniqueIdentifier for SegmentDWfe<E> {
    type DataType = Vec<(f64, f64)>;
    const PORT: u32 = 55_003;
}

/// Source segment wavefront error RMS `7x[m]`
#[derive(UID)]
#[uid(port = 55_004)]
pub enum SegmentWfeRms<const E: i32 = 0> {}

/// Source segment piston `7x[m]`
#[derive(UID)]
#[uid(port = 55_005)]
pub enum SegmentPiston<const E: i32 = 0> {}
#[derive(UID)]
#[uid(port = 55_005)]
pub enum SegmentD7Piston<const E: i32 = 0> {}

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
