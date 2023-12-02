/*!
# Scope client (`default` feature)

The scopes defined in the client module receive data from the scope servers
and show a live display of the data.

## Example

```ignore
use gmt_dos_clients_scope::client;

#[derive(gmt_dos_clients::interface::UID)]
pub enum Signal {}

let server_ip = "127.0.0.1";
let server_port = 5001;
let client_address = "127.0.0.1:0";

gmt_dos_clients_scope::client::Scope::new(server_ip, client_address)
    .signal::<Signal>(server_port).unwrap()
    .show();
```

*/

pub use gmt_dos_clients_scope::{
    client::{ClientError, GmtShot, GridScope, Scope, Shot, XScope},
    GmtScope, ImageScope, ImageScopeKind, PlotScope, ScopeKind,
};
