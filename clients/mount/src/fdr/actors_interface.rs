use std::ptr;

use gmt_dos_clients::interface::{Data, Read, Size, Update, Write};
#[cfg(fem)]
use gmt_dos_clients_io::gmt_fem::inputs::OSSGIRTooth6F;
use gmt_dos_clients_io::mount::{MountEncoders, MountSetPoint, MountTorques};

use crate::Mount;

impl<'a> Size<MountEncoders> for Mount<'a> {
    fn len(&self) -> usize {
        14
    }
}
impl<'a> Read<MountEncoders> for Mount<'a> {
    fn read(&mut self, data: Data<MountEncoders>) {
        unsafe {
            ptr::copy_nonoverlapping(
                (**data).as_ptr(),
                self.control.inputs.Mount_FB.as_mut_ptr(),
                <Mount as Size<MountEncoders>>::len(self),
            )
        }
        unsafe {
            ptr::copy_nonoverlapping(
                (**data).as_ptr(),
                self.drive.inputs.Mount_drv_Po.as_mut_ptr(),
                14,
            )
        }
    }
}
impl<'a> Size<MountSetPoint> for Mount<'a> {
    fn len(&self) -> usize {
        3
    }
}
impl<'a> Read<MountSetPoint> for Mount<'a> {
    fn read(&mut self, data: Data<MountSetPoint>) {
        unsafe {
            ptr::copy_nonoverlapping(
                (**data).as_ptr(),
                self.control.inputs.Mount_SP.as_mut_ptr(),
                <Mount as Size<MountSetPoint>>::len(self),
            )
        }
    }
}
impl<'a> Update for Mount<'a> {
    fn update(&mut self) {
        self.control.step();
        unsafe {
            ptr::copy_nonoverlapping(
                self.control.outputs.Mount_cmd.as_ptr(),
                self.drive.inputs.Mount_cmd.as_mut_ptr(),
                3,
            )
        }
        self.drive.step();
    }
}
impl<'a> Size<MountTorques> for Mount<'a> {
    fn len(&self) -> usize {
        16
    }
}

impl<'a> Write<MountTorques> for Mount<'a> {
    fn write(&mut self) -> Option<Data<MountTorques>> {
        let mut data = vec![0f64; <Mount as Size<MountTorques>>::len(self)];
        unsafe {
            ptr::copy_nonoverlapping(
                self.drive.outputs.Mount_T.as_ptr(),
                data.as_mut_ptr(),
                data.len(),
            )
        };
        Some(data.into())
    }
}

#[cfg(fem)]
impl<'a> Write<OSSGIRTooth6F> for Mount<'a> {
    fn write(&mut self) -> Option<Data<OSSGIRTooth6F>> {
        let data = vec![self.drive.outputs.ToothCAxialFo];
        Some(data.into())
    }
}
