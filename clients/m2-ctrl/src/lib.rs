mod actors_interface;
pub use actors_interface::AsmSegmentInnerController;
mod segment_builder;
use gmt_fem::FemError;
use matio_rs::MatioError;
pub use segment_builder::SegmentBuilder;
mod calibration;
pub use calibration::{Calibration, DataSource, SegmentCalibration};

#[derive(Debug, thiserror::Error)]
pub enum M2CtrlError {
    #[error("failed to load data from matfile")]
    MatFile(#[from] MatioError),
    #[error("failed to compute the stiffness")]
    Stiffness,
    #[error("FEM error")]
    Fem(#[from] FemError),
    #[error("expect (file_name, vec[var_name]) data source, found other data source")]
    DataSourceMatFile,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    Bincode(#[from] bincode::Error),
}
pub type Result<T> = std::result::Result<T, M2CtrlError>;

pub struct Segment<const ID: u8> {}
