//! GMT mount control model

use crate::{
    io::{Consuming, Data, Producing},
    Updating,
};
use mount_ctrl::controller;
use mount_ctrl::drives;
use std::{ptr, sync::Arc};

pub struct Mount<'a> {
    drive: drives::Controller<'a>,
    control: controller::Controller<'a>,
}
impl<'a> Mount<'a> {
    pub fn new() -> Self {
        Self {
            drive: drives::Controller::new(),
            control: controller::Controller::new(),
        }
    }
}

pub enum MountEncoders {}
impl<'a> Consuming<Vec<f64>, MountEncoders> for Mount<'a> {
    fn consume(&mut self, data: Arc<Data<Vec<f64>, MountEncoders>>) {
        if let controller::U::MountFB(val) = &mut self.control.mount_fb {
            unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
        }
        if let drives::U::Mountpos(val) = &mut self.drive.mount_pos {
            unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
        }
    }
}
impl<'a> Updating for Mount<'a> {
    fn update(&mut self) {
        self.control.next();
        if let (controller::Y::Mountcmd(src), drives::U::Mountcmd(dst)) =
            (&self.control.mount_cmd, &mut self.drive.mount_cmd)
        {
            unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), dst.len()) }
        }
        self.drive.next();
    }
}
pub enum MountTorques {}
impl<'a> Producing<Vec<f64>, MountTorques> for Mount<'a> {
    fn produce(&self) -> Option<Arc<Data<Vec<f64>, MountTorques>>> {
        let drives::Y::MountT(val) = &self.drive.mount_t;
        let mut data = vec![0f64; val.len()];
        unsafe { ptr::copy_nonoverlapping(val.as_ptr(), data.as_mut_ptr(), data.len()) }
        Some(Arc::new(Data::new(data)))
    }
}
