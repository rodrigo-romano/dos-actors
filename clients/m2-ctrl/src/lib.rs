pub mod assembly;

#[cfg(feature = "serde")]
pub mod nodes;

mod actors_interface;
pub use actors_interface::AsmSegmentInnerController;

#[cfg(fem)]
mod calibration;
#[cfg(fem)]
pub use calibration::{Calibration, DataSource, SegmentCalibration};

pub mod preprocessor;
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
}
pub type Result<T> = std::result::Result<T, M2CtrlError>;
