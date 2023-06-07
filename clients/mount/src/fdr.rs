/*!
# GMT mount control model

A unified Actor client for both the mount controller and the mount drive models from the [mount-ctrl] crate.

# Example

Mount actor:
```
use gmt_dos_clients_mount::Mount;
use dos_actors::prelude::*;
let mut mount: Actor<_> = Mount::new().into();

```

[mount-ctrl]: https://docs.rs/mount-ctrl
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
