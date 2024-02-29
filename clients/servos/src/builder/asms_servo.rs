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
            AsmsServo::new()
                .facesheet(
                    asms_servo::Facesheet::new()
                        .filter_piston_tip_tilt()
                        .transforms("KLmodesGS36p90.mat"),
                )
                .reference_body(
                    asms_servo::ReferenceBody::new()
                ),
        )
        .build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
*/

use gmt_dos_clients_fem::{DiscreteStateSpace, ExponentialMatrix, StateSpaceError};

mod facesheet;
mod reference_body;
pub use facesheet::Facesheet;
pub use reference_body::ReferenceBody;

use facesheet::FacesheetError;

use super::Include;

#[derive(Debug, thiserror::Error)]
pub enum AsmsServoError {
    #[error("Failed to build the ASMS facesheets")]
    Facesheet(#[from] FacesheetError),
}

/// ASMS builder
///
///
/// The rigid body motions of the facesheet are removed per default.
/// If is not desirable to remove the rigid body motions of the facesheet,
/// the type parameter `F` can be set to `false`.
#[derive(Debug, Clone, Default)]
pub struct AsmsServo<const F: bool = true> {
    facesheet: Option<Facesheet<F>>,
    reference_body: Option<ReferenceBody>,
}

impl<const F: bool> AsmsServo<F> {
    /// Creates a new ASMS builder
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the ASMS [Facesheet] builder
    pub fn facesheet(mut self, facesheet: Facesheet<F>) -> Self {
        self.facesheet = Some(facesheet);
        self
    }
    /// Sets the ASMS [ReferenceBody] builder
    pub fn reference_body(mut self, reference_body: ReferenceBody) -> Self {
        self.reference_body = Some(reference_body);
        self
    }
    pub fn build(&mut self, fem: &gmt_fem::FEM) -> Result<(), AsmsServoError> {
        if F && self.reference_body.is_none() {
            self.reference_body = Some(ReferenceBody::new());
        }
        if let Some(facesheet) = self.facesheet.as_mut() {
            facesheet.build(fem)?;
        }
        Ok(())
    }
}

impl<'a, const F: bool> Include<'a, AsmsServo<F>> for DiscreteStateSpace<'a, ExponentialMatrix> {
    fn including(self, asms_servo: Option<&'a mut AsmsServo<F>>) -> Result<Self, StateSpaceError> {
        let Some(asms_servo) = asms_servo else {
            return Ok(self);
        };
        self.including(asms_servo.facesheet.as_mut())?
            .including(asms_servo.reference_body.as_mut())
    }
}
