pub mod assembly;
mod calibration;
pub mod subsystems;
pub mod systems;

pub use calibration::Calibration;

pub enum M1<const ACTUATOR_RATE: usize> {}
impl<const ACTUATOR_RATE: usize> M1<ACTUATOR_RATE> {
    pub fn new(
        calibration: &Calibration,
    ) -> anyhow::Result<gmt_dos_actors::system::Sys<assembly::M1<ACTUATOR_RATE>>> {
        Ok(
            gmt_dos_actors::system::Sys::new(assembly::M1::<ACTUATOR_RATE>::new(calibration)?)
                .build()?,
        )
    }
}
