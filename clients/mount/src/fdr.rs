/*!
# GMT mount control model

A [gmt_dos-actors] client for the GMT mount control system.
*/

use std::marker::PhantomData;

use gmt_mount_ctrl_controller::MountController;
use gmt_mount_ctrl_driver::MountDriver;

mod actors_interface;

/// Mount control system
pub struct Mount<'a> {
    drive: MountDriver,
    control: MountController,
    phantom: PhantomData<&'a MountDriver>,
}
impl<'a> Mount<'a> {
    /// Returns the mount controller
    pub fn new() -> Self {
        Self {
            drive: MountDriver::new(),
            control: MountController::new(),
            phantom: PhantomData,
        }
    }
}
