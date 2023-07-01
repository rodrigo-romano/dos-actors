//! # GMT DOS Clients IO
//!
//! Definitions of the types for the inputs and the ouputs of [gmt_dos-actors](https://crates.io/crates/gmt_dos-actors)
//! clients used with the GMT Integrated Model

pub mod gmt_m1;
pub mod gmt_m2;

/// Mount
pub mod mount {
    use gmt_dos_clients::interface::UID;
    /// Mount Encoders
    #[derive(UID)]
    pub enum MountEncoders {}
    /// Mount Torques
    #[derive(UID)]
    pub enum MountTorques {}
    /// Mount set point
    #[derive(UID)]
    pub enum MountSetPoint {}
}
/// CFD wind loads
pub mod cfd_wind_loads {
    use gmt_dos_clients::interface::UID;
    /// CFD Mount Wind Loads
    #[derive(UID)]
    pub enum CFDMountWindLoads {}
    /// CFD M1 Loads
    #[derive(UID)]
    pub enum CFDM1WindLoads {}
    /// CFD M2 Wind Loads
    #[derive(UID)]
    pub enum CFDM2WindLoads {}
}

pub mod optical_metrics {
    use gmt_dos_clients::interface::UID;
    /// Focal plane tip-tilt
    #[derive(UID)]
    pub enum TipTilt {}
    /// Pupil plane segment tip-tilt
    #[derive(UID)]
    pub enum SegmentTipTilt {}
    /// Pupil plane segment piston
    #[derive(UID)]
    pub enum SegmentPiston {}
}
