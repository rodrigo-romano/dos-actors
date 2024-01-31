//! # GMT Servo-Mechanisms
//!
//! A dos-actors [system] that combines together a few clients:
//!   * the GMT [FEM]
//!   * the GMT [mount] control system
//!   * the GMT [M1] control system
//!   * the GMT [M2] control system
//!
//! [system]: https://docs.rs/gmt_dos-actors/latest/gmt_dos_actors/system
//! [FEM]: https://docs.rs/gmt_dos-clients_fem/latest/gmt_dos_clients_fem/
//! [mount]: https://docs.rs/gmt_dos-clients_mount/latest/gmt_dos_clients_mount/
//! [M1]: https://docs.rs/gmt_dos-clients_m1-ctrl/latest/gmt_dos_clients_m1_ctrl/
//! [M2]: https://docs.rs/gmt_dos-clients_m2-ctrl/latest/gmt_dos_clients_m2_ctrl/
//!
use gmt_dos_actors::system::Sys;

mod servos;

/// GMT servo-mechanisms client
pub enum GmtServoMechanisms<const M1_RATE: usize, const M2_RATE: usize = 1> {}
impl<const M1_RATE: usize, const M2_RATE: usize> GmtServoMechanisms<M1_RATE, M2_RATE> {
    pub fn new(
        sim_sampling_frequency: f64,
        fem: gmt_fem::FEM,
    ) -> anyhow::Result<Sys<servos::GmtServoMechanisms<'static, M1_RATE, M2_RATE>>> {
        Ok(Sys::new(
            servos::GmtServoMechanisms::<'static, M1_RATE, M2_RATE>::new(
                sim_sampling_frequency,
                fem,
            )?,
        )
        .build()?)
    }
}

/// GMT FEM client
pub type GmtFem = gmt_dos_clients_fem::DiscreteModalSolver<gmt_dos_clients_fem::ExponentialMatrix>;
/// GMT M1 client
pub type GmtM1 = gmt_dos_clients_m1_ctrl::assembly::DispatchIn;
/// GMT mount client
pub type GmtMount<'a> = gmt_dos_clients_mount::Mount<'a>;
/// GMT M2 positioners client
pub type GmtM2Hex = gmt_dos_clients_m2_ctrl::positioner::AsmsPositioners;
/// GMT M2 mirror client
pub type GmtM2 = gmt_dos_clients_m2_ctrl::assembly::DispatchIn;
