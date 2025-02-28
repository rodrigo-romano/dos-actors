/*!
# ASMS Control Systems

Models of the control systems of the ASMS positioners and voice coil actuators

## Example

```no_run
use gmt_dos_actors::system::Sys;
use gmt_dos_clients_m2_ctrl::AsmsPositioners;
use gmt_dos_systems_m2::ASMS;
use gmt_fem::FEM;

let mut fem = FEM::from_env()?;
let positioners = AsmsPositioners::new(&mut fem)?;
let asms: Sys<ASMS> = ASMS::new(&mut fem)?.build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

 */

#[cfg(topend = "ASM")]
mod asms;
#[cfg(topend = "FSM")]
mod fsms;

#[cfg(feature = "serde")]
pub mod nodes;


#[derive(Debug, thiserror::Error)]
pub enum M2Error {
    #[error("failed to load data from matfile")]
    MatFile(#[from] matio_rs::MatioError),
    #[error("failed to compute the stiffness")]
    Stiffness,
    #[error("FEM error")]
    Fem(#[from] gmt_fem::FemError),
    #[error("expected (file_name, vec[var_name]) data source, found other data source")]
    DataSourceMatFile,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[cfg(feature = "bincode")]
    #[error(transparent)]
    Encode(#[from] bincode::error::EncodeError),
    #[cfg(feature = "bincode")]
    #[error(transparent)]
    Decode(#[from] bincode::error::DecodeError),
    #[error("expected matrix size {0:?}, found {1:?}")]
    MatrixSizeMismatch((usize, usize), (usize, usize)),
    #[error("failed to inverse ASMS stiffness matrices")]
    InverseStiffness,
}
