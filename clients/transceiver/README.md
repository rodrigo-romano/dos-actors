# GMT DOS Actors Transceiver

The `gmt_dos-clients_transceiver` provides implementation for two GMT DOS actors clients: a transmitter
and a receiver allowing to transfer data between GMT DOS actors models through the network.

The communication betweem the transmitter and the receiver is secured by procuring a signed certificate
shared by both the transmitter and the receiver and a private key for the transmitter only.

The certificate and the private key are generated with
`
cargo run --bin crypto
`