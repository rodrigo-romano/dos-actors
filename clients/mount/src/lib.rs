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

#[cfg(fem)]
mod builder;
#[cfg(fem)]
pub use builder::Builder;

#[cfg(not(feature = "mount-fdr"))]
mod pdr;
#[cfg(not(feature = "mount-fdr"))]
pub use pdr::Mount;

#[cfg(feature = "mount-fdr")]
mod fdr;
#[cfg(feature = "mount-fdr")]
pub use fdr::Mount;
