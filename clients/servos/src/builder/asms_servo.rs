/*!
Repository for the ASMS component builders

## Example

```no_run
use gmt_dos_clients_servos::{asms_servo, AsmsServo, GmtServoMechanisms};
use gmt_fem::FEM;

const ACTUATOR_RATE: usize = 80; // 100Hz

let frequency = 8000_f64; // Hz
let fem = FEM::from_env()?;

let gmt_servos =
    GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
        .asms_servo(
            AsmsServo::new().facesheet(
                asms_servo::Facesheet::new()
                    .filter_piston_tip_tilt()
                    .transforms("KLmodesGS36p90.mat"),
            ),
        )
        .build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
*/

mod facesheet;
pub use facesheet::Facesheet;
use gmt_dos_clients_fem::{DiscreteStateSpace, ExponentialMatrix, StateSpaceError};

use self::facesheet::FacesheetError;

use super::Include;

#[derive(Debug, thiserror::Error)]
pub enum AsmsServoError {
    #[error("Failed to build the ASMS facesheets")]
    Facesheet(#[from] FacesheetError),
}

/// ASMS builder
#[derive(Debug, Clone, Default)]
pub struct AsmsServo {
    facesheet: Option<Facesheet>,
}

impl AsmsServo {
    /// Creates a new ASMS builder
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the ASMS [Facesheet] builder
    pub fn facesheet(mut self, facesheet: Facesheet) -> Self {
        self.facesheet = Some(facesheet);
        self
    }
    pub fn build(&mut self, fem: &gmt_fem::FEM) -> Result<(), AsmsServoError> {
        if let Some(facesheet) = self.facesheet.as_mut() {
            facesheet.build(fem)?;
        }
        Ok(())
    }
}

impl<'a> Include<'a, AsmsServo> for DiscreteStateSpace<'a, ExponentialMatrix> {
    fn including(self, asm_servo: Option<&'a mut AsmsServo>) -> Result<Self, StateSpaceError> {
        let Some(AsmsServo {
            facesheet: Some(facesheet),
        }) = asm_servo
        else {
            return Ok(self);
        };
        Ok(if let Some(transforms) = facesheet.transforms_view() {
            self.outs_with_by_name(
                (1..=7).map(|i| format!("M2_segment_{i}_axial_d")).collect(),
                transforms,
            )?
        } else {
            self.outs_by_name((1..=7).map(|i| format!("M2_segment_{i}_axial_d")).collect())?
        })
    }
}
