use mount_ctrl::{controller, drives};

mod actors_interface;

/// Mount control system
pub struct Mount<'a> {
    drive: drives::Controller<'a>,
    control: controller::Controller<'a>,
}
impl<'a> Mount<'a> {
    /// Returns the mount controller
    pub fn new() -> Self {
        Self {
            drive: drives::Controller::new(),
            control: controller::Controller::new(),
        }
    }
}
