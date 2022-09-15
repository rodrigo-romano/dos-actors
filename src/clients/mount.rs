/*!
# GMT mount control model

A unified Actor client for both the mount controller and the mount drive models from the [mount-ctrl] crate.

*The [Mount] client is enabled with the `mount-ctrl` feature.*

# Example

Mount actor:
```
use dos_actors::clients::mount::Mount;
use dos_actors::prelude::*;
let mut mount: Actor<_> = Mount::new().into();

```

[mount-ctrl]: https://docs.rs/mount-ctrl
*/

use crate::{
    io::{Data, Read, Write},
    Size, Update,
};
use mount_ctrl::{controller, drives, ControllerController, DriveController};
use std::{ptr, sync::Arc};
use uid_derive::UID;

pub struct Mount<'a> {
    drive: drives::Controller<'a>,
    control: controller::Controller<'a>,
}
impl<'a> Mount<'a> {
    /// Returns a default mount controller
    pub fn new() -> Self {
        Self {
            drive: drives::Controller::new(),
            control: controller::Controller::new(),
        }
    }
}

#[derive(UID)]
pub enum MountEncoders {}
impl<'a> Size<MountEncoders> for Mount<'a> {
    fn len(&self) -> usize {
        14
    }
}
impl<'a> Read<MountEncoders> for Mount<'a> {
    fn read(&mut self, data: Arc<Data<MountEncoders>>) {
        if let Some(val) = &mut self.control.mount_fb() {
            assert_eq!(
                data.len(),
                val.len(),
                "data size ({}) do not match MountFb size ({})",
                data.len(),
                val.len()
            );
            unsafe { ptr::copy_nonoverlapping((**data).as_ptr(), val.as_mut_ptr(), val.len()) }
        }
        if let Some(val) = &mut self.drive.mount_pos() {
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
#[derive(UID)]
pub enum MountSetPoint {}
impl<'a> Size<MountSetPoint> for Mount<'a> {
    fn len(&self) -> usize {
        3
    }
}
impl<'a> Read<MountSetPoint> for Mount<'a> {
    fn read(&mut self, data: Arc<Data<MountSetPoint>>) {
        if let Some(val) = &mut self.control.mount_sp() {
            assert_eq!(
                data.len(),
                val.len(),
                "data size ({}) do not match MountFb size ({})",
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
        if let (Some(src), Some(dst)) = (&self.control.mount_cmd(), &mut self.drive.mount_cmd()) {
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
#[derive(UID)]
pub enum MountTorques {}
impl<'a> Size<MountTorques> for Mount<'a> {
    fn len(&self) -> usize {
        20
    }
}
impl<'a> Write<MountTorques> for Mount<'a> {
    fn write(&mut self) -> Option<Arc<Data<MountTorques>>> {
        self.drive.mount_t().as_ref().map(|val| {
            let mut data = vec![0f64; val.len()];
            unsafe { ptr::copy_nonoverlapping(val.as_ptr(), data.as_mut_ptr(), data.len()) };
            Arc::new(Data::new(data))
        })
    }
}
