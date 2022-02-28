//! GMT mount control model

use crate::{
    io::{Data, Read, Write},
    Update,
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
impl<'a> Read<Vec<f64>, MountEncoders> for Mount<'a> {
    fn read(&mut self, data: Arc<Data<Vec<f64>, MountEncoders>>) {
        if let controller::U::MountFB(val) = &mut self.control.mount_fb {
            assert_eq!(
                data.len(),
                val.len(),
                "data size ({}) do not match MountFb size ({})",
                data.len(),
                val.len()
            );
            unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
        }
        if let drives::U::Mountpos(val) = &mut self.drive.mount_pos {
            assert_eq!(
                data.len(),
                val.len(),
                "data size ({}) do not match Mountpos size ({})",
                data.len(),
                val.len()
            );
            unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
        }
    }
}
impl<'a> Update for Mount<'a> {
    fn update(&mut self) {
        self.control.next();
        if let (controller::Y::Mountcmd(src), drives::U::Mountcmd(dst)) =
            (&self.control.mount_cmd, &mut self.drive.mount_cmd)
        {
            assert_eq!(
                src.len(),
                dst.len(),
                "control.mount_cmd size ({}) do not match drive.mount_cmd size ({})",
                src.len(),
                dst.len()
            );
            unsafe { ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), dst.len()) }
        }
        self.drive.next();
    }
}
pub enum MountTorques {}
impl<'a> Write<Vec<f64>, MountTorques> for Mount<'a> {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, MountTorques>>> {
        let drives::Y::MountT(val) = &self.drive.mount_t;
        let mut data = vec![0f64; val.len()];
        unsafe { ptr::copy_nonoverlapping(val.as_ptr(), data.as_mut_ptr(), data.len()) }
        Some(Arc::new(Data::new(data)))
    }
}
