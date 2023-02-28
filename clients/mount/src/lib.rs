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

use mount_ctrl::{controller, drives};

mod actors_interface;
mod builder;
pub use builder::Builder;

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
