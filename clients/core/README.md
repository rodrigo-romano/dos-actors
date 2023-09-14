# `gmt_dos-clients`

[![Crates.io](https://img.shields.io/crates/v/gmt_dos-clients.svg)](https://crates.io/crates/gmt_dos-clients)
[![Documentation](https://docs.rs/gmt_dos-clients/badge.svg)](https://docs.rs/gmt_dos-clients/)

The crate defines the client-to-actor [interface](https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/interface/index.html) and a set of "generic" [clients](https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/).

If the crate is required solely to implement the interface for a new client, add it to the list of dependencies like so:

```shell
cargo add --no-default-features --features interface gmt_dos-clients
```