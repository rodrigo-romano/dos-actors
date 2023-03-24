# Control Systems

The clients for the different GMT control systems are all implemented into different crates.
These crates implement the client interfaces to `dos-actors` for crates that are wrappers around C implementation of the control systems.
The C implementations are themselves generated from Simulink control models. 

| GMT | Crate ||||
|-|-|-|-|-|
| Mount Control |`gmt_dos-clients_mount`| [crates.io](https://crates.io/crates/gmt_dos-clients_mount) | [docs.rs](https://docs.rs/gmt_dos-clients_mount) | [github](https://github.com/rconan/dos-actors/tree/main/clients/mount) |
| M1 Control |`gmt_dos-clients_m1-ctrl`| [crates.io](https://crates.io/crates/gmt_dos-clients_m1-ctrl) | [docs.rs](https://docs.rs/gmt_dos-clients_m1-ctrl) | [github](https://github.com/rconan/dos-actors/tree/main/clients/m1-ctrl) |
| M2 Control |`gmt_dos-clients_m2-ctrl`| [crates.io](https://crates.io/crates/gmt_dos-clients_m2-ctrl) | [docs.rs](https://docs.rs/gmt_dos-clients_m2-ctrl) | [github](https://github.com/rconan/dos-actors/tree/main/clients/m2-ctrl) |
