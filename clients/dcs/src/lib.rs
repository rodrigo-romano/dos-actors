mod pk_sys_types;
pub use pk_sys_types::ImMountDemands;
mod connector;
pub use connector::Connector;
mod dcs_data;
pub use dcs_data::DcsData;
mod mount_trajectory;
pub use mount_trajectory::{MountTrajectory, OcsMountTrajectory};
mod dcs;
pub use dcs::Dcs;

#[derive(Debug, thiserror::Error)]
pub enum DcsError {
    #[error("Failed to connect")]
    Nanomsg(#[from] nanomsg::result::Error),
    #[error("Failed to deserialize")]
    Deserialization(#[from] rmp_serde::decode::Error),
    #[error("Failed to serialize")]
    Serialization(#[from] rmp_serde::encode::Error),
}

pub trait DcsProtocol {}
pub enum Push {}
impl DcsProtocol for Push {}
pub enum Pull {}
impl DcsProtocol for Pull {}
