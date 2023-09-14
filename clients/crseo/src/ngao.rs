//! # Natural Guide Star Adaptive Optics
//!
//! Integrated model of the NGAO Observing Performance Mode of the GMT

mod optical_model;
use gmt_dos_clients::interface::UID;
use gmt_dos_clients_io::optics::M2modes;
pub use optical_model::{GmtWavefront, OpticalModel};

mod wavefront_sensor;
pub use wavefront_sensor::{
    Frame, GuideStar, PistonMode, SensorData, ShackHartmann, WavefrontSensor,
};

// mod sensor_fusion;
// pub use sensor_fusion::{HdfsIntegrator, HdfsOrNot, PwfsIntegrator};

#[derive(UID)]
pub enum ResidualPistonMode {}

#[derive(UID)]
#[alias(name = M2modes, client = OpticalModel, traits = Read)]
pub enum ResidualM2modes {}

#[derive(UID)]
pub enum M1Rxy {}
