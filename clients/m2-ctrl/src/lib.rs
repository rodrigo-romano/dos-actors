mod actors_interface;
#[cfg(feature = "serde")]
pub mod nodes;
mod subsystem;
pub use actors_interface::AsmSegmentInnerController;
pub use subsystem::ASMS;
#[cfg(fem_with_asm)]
mod segment_builder;
use gmt_fem::FemError;
use matio_rs::MatioError;
#[cfg(fem_with_asm)]
pub use segment_builder::SegmentBuilder;
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
    MatFile(#[from] MatioError),
    #[error("failed to compute the stiffness")]
    Stiffness,
    #[error("FEM error")]
    Fem(#[from] FemError),
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

#[allow(dead_code)]
pub const M2S1: u8 = 1;
#[allow(dead_code)]
pub const M2S2: u8 = 2;
#[allow(dead_code)]
pub const M2S3: u8 = 3;
#[allow(dead_code)]
pub const M2S4: u8 = 4;
#[allow(dead_code)]
pub const M2S5: u8 = 5;
#[allow(dead_code)]
pub const M2S6: u8 = 6;
#[allow(dead_code)]
pub const M2S7: u8 = 7;

pub const fn m2s(i: u8) -> u8 {
    i
}

pub struct Segment<const ID: u8> {}

#[macro_export]
macro_rules! segment {
    ($sid:tt,$plant:expr,$($args:expr),*) => {
        match $sid {
            i if i == 1 => gmt_dos_actors::model!(Segment::<{$crate::M2S1}>::builder($($args),*).build($plant)?),
            i if i == 2 => gmt_dos_actors::model!(Segment::<{$crate::M2S2}>::builder($($args),*).build($plant)?),
            i if i == 3 => gmt_dos_actors::model!(Segment::<{$crate::M2S3}>::builder($($args),*).build($plant)?),
            i if i == 4 => gmt_dos_actors::model!(Segment::<{$crate::M2S4}>::builder($($args),*).build($plant)?),
            i if i == 5 => gmt_dos_actors::model!(Segment::<{$crate::M2S5}>::builder($($args),*).build($plant)?),
            i if i == 6 => gmt_dos_actors::model!(Segment::<{$crate::M2S6}>::builder($($args),*).build($plant)?),
            i if i == 7 => gmt_dos_actors::model!(Segment::<{$crate::M2S7}>::builder($($args),*).build($plant)?),
            __ => unimplemented!(),
        }
    };
}

#[macro_export]
macro_rules! M2S {
    ($sid:literal) => {
        paste::paste! {[< { $crate::M2S $sid } >]}
    };
}
