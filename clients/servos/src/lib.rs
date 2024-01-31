use gmt_dos_actors::system::Sys;

mod servos;
// pub use servos::GmtServoMechanisms;

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

pub type GmtFem = gmt_dos_clients_fem::DiscreteModalSolver<gmt_dos_clients_fem::ExponentialMatrix>;
pub type GmtM1 = gmt_dos_clients_m1_ctrl::assembly::DispatchIn;
pub type GmtMount<'a> = gmt_dos_clients_mount::Mount<'a>;
pub type GmtM2Hex = gmt_dos_clients_m2_ctrl::positioner::AsmsPositioners;
pub type GmtM2 = gmt_dos_clients_m2_ctrl::assembly::DispatchIn;
