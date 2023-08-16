# `gmt_dos-clients_transceiver`

[![Crates.io](https://img.shields.io/crates/v/gmt_dos-clients_transceiver.svg)](https://crates.io/crates/gmt_dos-clients_transceiver)
[![Documentation](https://docs.rs/gmt_dos-clients_transceiver/badge.svg)](https://docs.rs/gmt_dos-clients_transceiver/)

The `gmt_dos-clients_transceiver` provides implementation for two GMT DOS actors clients: a transmitter
and a receiver allowing to transfer data between GMT DOS actors models through the network.

The communications between the transmitter and the receiver are secured by procuring a signed certificate
shared by both the transmitter and the receiver and a private key for the transmitter only.

The certificate and the private key are generated with
`
cargo run --bin crypto
`