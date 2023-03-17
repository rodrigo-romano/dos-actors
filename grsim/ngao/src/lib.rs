//! # Natural Guide Star Adaptive Optics
//!
//! Integrated model of the NGAO Observing Performance Mode of the GMT

mod optical_model;
pub use optical_model::LittleOpticalModel;

mod wavefront_sensor;
pub use wavefront_sensor::{GuideStar, PistonMode, SensorData, WavefrontSensor};

mod sensor_fusion;
pub use sensor_fusion::{
    HdfsIntegrator, HdfsOrNot, PwfsIntegrator, ResidualM2modes, ResidualPistonMode,
};
