/*!
# GMT DOS Actors Scope

`gmt_dos-clients_scope` acquire signals from a [transmitter](gmt_dos_clients_transceiver::Transceiver)
and display them graphically.

The communication between the transmitter and the scope is secured with a signed certificate
that must be provided by the transmitter.

## Examples

```
let transmitter_address = "127.0.0.1:5001";
let scope_address = "127.0.0.1:5000";
Scope::new(transmitter_address,scope_address)
    .signal::<S1>(1e-3).unwrap()
    .signal::<S2>(1e-1).unwrap()
    .show();
```

*/

mod scope;
pub use scope::Scope;
