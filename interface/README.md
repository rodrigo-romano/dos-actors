# `gmt_dos-actors-clients_interface`

[![Crates.io](https://img.shields.io/crates/v/gmt_dos-actors-clients_interface.svg)](https://crates.io/crates/gmt_dos-actors-clients_interface)
[![Documentation](https://docs.rs/gmt_dos-actors-clients_interface/badge.svg)](https://docs.rs/gmt_dos-actors-clients_interface/)

Interface definition betweeen an [actor] and an [actor]'s client.

Data is passed from the [actor] to the client by invoking `Read::read` from the client.

Data is passed from the client to the [actor] by invoking `Write::write` from the client.

The client state may be updated by invoking `Update::update` from the client

[actor]: https://docs.rs/gmt_dos-actors