use gmt_dos_actors::system::SystemError;
pub use gmt_dos_clients_m1_ctrl::Calibration;
use gmt_fem::FEM;

pub mod assembly;
pub mod subsystems;
pub mod systems;

pub enum M1<const ACTUATOR_RATE: usize> {}
impl<const ACTUATOR_RATE: usize> M1<ACTUATOR_RATE> {
    pub fn new(
        fem: &mut FEM,
    ) -> Result<gmt_dos_actors::system::Sys<assembly::M1<ACTUATOR_RATE>>, SystemError> {
        let calibration = Calibration::new(fem);
        Ok(
            gmt_dos_actors::system::Sys::new(assembly::M1::<ACTUATOR_RATE>::new(&calibration)?)
                .build()?,
        )
    }
}
