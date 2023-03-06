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

#[allow(dead_code)]
pub const S1: u8 = 1;
#[allow(dead_code)]
pub const S2: u8 = 2;
#[allow(dead_code)]
pub const S3: u8 = 3;
#[allow(dead_code)]
pub const S4: u8 = 4;
#[allow(dead_code)]
pub const S5: u8 = 5;
#[allow(dead_code)]
pub const S6: u8 = 6;
#[allow(dead_code)]
pub const S7: u8 = 7;

pub struct Segment<const ID: u8> {}

#[macro_export]
macro_rules! segment {
    ($sid:tt,$plant:expr,$($args:expr),*) => {
        match $sid {
            i if i == 1 => gmt_dos_actors::model!(Segment::<{$crate::S1}>::builder($($args),*).build($plant)?),
            i if i == 2 => gmt_dos_actors::model!(Segment::<{$crate::S2}>::builder($($args),*).build($plant)?),
            i if i == 3 => gmt_dos_actors::model!(Segment::<{$crate::S3}>::builder($($args),*).build($plant)?),
            i if i == 4 => gmt_dos_actors::model!(Segment::<{$crate::S4}>::builder($($args),*).build($plant)?),
            i if i == 5 => gmt_dos_actors::model!(Segment::<{$crate::S5}>::builder($($args),*).build($plant)?),
            i if i == 6 => gmt_dos_actors::model!(Segment::<{$crate::S6}>::builder($($args),*).build($plant)?),
            i if i == 7 => gmt_dos_actors::model!(Segment::<{$crate::S7}>::builder($($args),*).build($plant)?),
            __ => unimplemented!(),
        }
    };
}
