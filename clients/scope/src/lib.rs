/*!
# GMT DOS Actors Scope

`gmt_dos-clients_scope` acquire signals from a [transmitter](gmt_dos_clients_transceiver::Transceiver)
and display them graphically.

The communication between the transmitter and the scope is secured with a signed certificate
that must be provided by the transmitter.

## Examples

```
let transmitter_ip = "127.0.0.1";
let transmitter_port = 5001;
let scope_address = "127.0.0.1:0";
Scope::new(transmitter_ip,scope_address)
    .signal::<S1>(transmitter_port).unwrap()
    .show();
```

*/

mod payload;

#[cfg(not(feature = "server"))]
mod scope;
#[cfg(not(feature = "server"))]
pub use gmt_dos_clients_scope_macros::scope;
#[cfg(not(feature = "server"))]
pub use scope::{ImageScope, PlotScope, Scope, ScopeError, ScopeKind, Shot, XScope};

mod server;
#[cfg(feature = "server")]
pub use server::*;
