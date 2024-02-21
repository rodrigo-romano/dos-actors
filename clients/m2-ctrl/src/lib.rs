pub mod assembly;

#[cfg(feature = "serde")]
pub mod nodes;

mod actors_interface;
pub mod positioner;

pub use actors_interface::AsmSegmentInnerController;

#[cfg(fem)]
mod calibration;
#[cfg(fem)]
pub use calibration::{Calibration, DataSource, SegmentCalibration};

pub mod preprocessor;
use gmt_dos_clients_fem::{Model, Switch};
use gmt_dos_clients_io::Assembly;
use gmt_fem::FEM;
#[doc(inline)]
pub use preprocessor::Preprocessor;

#[derive(Debug, thiserror::Error)]
pub enum M2CtrlError {
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
pub type Result<T> = std::result::Result<T, M2CtrlError>;

pub enum ASMS<const R: usize = 1> {}
impl<const R: usize> ASMS<R> {
    pub fn new(
        n_mode: Vec<usize>,
        ks: Vec<Option<Vec<f64>>>,
    ) -> anyhow::Result<gmt_dos_actors::system::Sys<assembly::ASMS<R>>> {
        Ok(gmt_dos_actors::system::Sys::new(assembly::ASMS::<R>::new(n_mode, ks)).build()?)
    }
    pub fn from_fem(
        fem: &mut FEM,
        n_mode: Option<Vec<usize>>,
    ) -> anyhow::Result<gmt_dos_actors::system::Sys<assembly::ASMS<R>>> {
        let mut vc_f2d = vec![];
        for i in 1..=7 {
            fem.switch_inputs(Switch::Off, None)
                .switch_outputs(Switch::Off, None);

            vc_f2d.push(
                fem.switch_inputs_by_name(vec![format!("MC_M2_S{i}_VC_delta_F")], Switch::On)
                    .and_then(|fem| {
                        fem.switch_outputs_by_name(
                            vec![format!("MC_M2_S{i}_VC_delta_D")],
                            Switch::On,
                        )
                    })
                    .map(|fem| {
                        fem.reduced_static_gain()
                            .unwrap_or_else(|| fem.static_gain())
                    })?,
            );
        }
        fem.switch_inputs(Switch::On, None)
            .switch_outputs(Switch::On, None);

        let ks: Vec<_> = vc_f2d
            .into_iter()
            .map(|x| {
                Ok::<Vec<f64>, M2CtrlError>(
                    x.try_inverse()
                        .ok_or(M2CtrlError::InverseStiffness)?
                        .as_slice()
                        .to_vec(),
                )
            })
            .collect::<Result<Vec<Vec<f64>>>>()?;
        Ok(Self::new(
            n_mode.unwrap_or(vec![675; <assembly::ASMS as Assembly>::N]),
            ks.into_iter().map(|ks| Some(ks)).collect(),
        )?)
    }
}
