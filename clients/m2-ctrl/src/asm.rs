/*!
# ASMS Control Systems

Models of the control systems of the ASMS positioners and voice coil actuators

## Example

```no_run
use gmt_dos_actors::system::Sys;
use gmt_dos_clients_m2_ctrl::Positioners;
use gmt_fem::FEM;

let mut fem = FEM::from_env()?;
let positioners = Positioners::new(&mut fem)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

 */

#[cfg(feature = "serde")]
pub mod nodes;

mod controller;
pub use controller::AsmSegmentInnerController;

pub mod preprocessor;
pub use preprocessor::Preprocessor;
