/*!
# Scope client (`client` feature)

The scopes defined in the client module receive data from the scope servers
and show a live display of the data.

## Example

```ignore
use gmt_dos_clients_scope::client;

#[derive(gmt_dos_clients::interface::UID)]
#[uid(port = 5001)]
pub enum Signal {}

gmt_dos_clients_scope::client::Scope::new()
    .signal::<Signal>().unwrap()
    .show();
```

*/

mod scope;
pub use scope::{ClientError, GmtShot, Scope, Shot, XScope};
mod gridscope;
pub use gridscope::GridScope;
