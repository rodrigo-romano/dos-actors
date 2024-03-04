/*!
# GMT mount control model

A [gmt_dos-actors] client for the GMT mount control system.
*/

use serde::{Deserialize, Serialize};

use gmt_mount_ctrl_controller::MountController;
use gmt_mount_ctrl_driver::MountDriver;

mod actors_interface;

/// Mount control system
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Mount {
    drive: MountDriver,
    control: MountController,
}
impl Mount {
    /// Returns the mount controller
    pub fn new() -> Self {
        Self {
            drive: MountDriver::new(),
            control: MountController::new(),
        }
    }
}
