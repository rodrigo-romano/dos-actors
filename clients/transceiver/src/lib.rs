/*!
# GMT DOS Actors Transceiver

The `gmt_dos-clients_transceiver` provides implementation for two GMT DOS actors clients: a [Transmitter]
and a [Receiver] allowing to transfer [Data] between GMT DOS actors models through the network.

The communication betweem the transmitter and the receiver is secured by procuring a signed certificate
shared by both the transmitter and the receiver and a private key for the transmitter only (see also [Crypto]).

The certificate and the private key are generated with
`
cargo run --bin crypto
`

[Data]: https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/interface/struct.Data.html
*/

mod crypto;
mod monitor;
mod receiver;
mod transmitter;

use std::marker::PhantomData;

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update, Write};
use quinn::Endpoint;

pub use crypto::Crypto;
pub use monitor::Monitor;

#[derive(Debug, thiserror::Error)]
pub enum TransceiverError {
    #[error("failed to parse IP socket address")]
    Socket(#[from] std::net::AddrParseError),
    // #[error("failed to bind endpoint to socket address")]
    // IO(#[from] std::io::Result<Endpoint>),
    #[error("connection failed")]
    ConnectionError(#[from] quinn::ConnectionError),
    #[error("failed to connect")]
    ConnectError(#[from] quinn::ConnectError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("encryption failed")]
    Crypto(#[from] rustls::Error),
    #[error("failed to send data to receiver")]
    SendToRx(#[from] quinn::WriteError),
    #[error("data serialization failed ({0})")]
    Encode(String),
    #[error("data deserialization failed ({0})")]
    Decode(String),
    #[error("failed to read data from transmitter")]
    RecvFromTx(#[from] quinn::ReadToEndError),
    #[error("failed to join task")]
    Join(#[from] tokio::task::JoinError),
}
pub type Result<T> = std::result::Result<T, TransceiverError>;

/// Receiver functionality of a [Transceiver]
pub enum Receiver {}
/// Transmitter functionality of a [Transceiver]
pub enum Transmitter {}
/// [Transceiver] without purpose
pub enum Unset {}
trait RxOrTx {}
impl RxOrTx for Transmitter {}
impl RxOrTx for Receiver {}

pub enum On {}
pub enum Off {}

/// Transmitter and receiver of [gmt_dos-actors](https://docs.rs/gmt_dos-actors/) [Data](https://docs.rs/gmt_dos-clients/latest/gmt_dos_clients/interface/struct.Data.html)
#[derive(Debug)]
pub struct Transceiver<U: UniqueIdentifier, F = Unset, S = Off> {
    crypto: Crypto,
    endpoint: Option<quinn::Endpoint>,
    server_address: String,
    tx: Option<flume::Sender<Data<U>>>,
    rx: Option<flume::Receiver<Data<U>>>,
    function: PhantomData<F>,
    state: PhantomData<S>,
}
impl<U: UniqueIdentifier, F> Transceiver<U, F> {
    pub fn new<S: Into<String>>(crypto: Crypto, server_address: S, endpoint: Endpoint) -> Self {
        let (tx, rx) = flume::unbounded();
        Self {
            crypto,
            server_address: server_address.into(),
            endpoint: Some(endpoint),
            tx: Some(tx),
            rx: Some(rx),
            function: PhantomData,
            state: PhantomData,
        }
    }
}

impl<U: UniqueIdentifier, V: UniqueIdentifier, F> From<&Transceiver<U, F>> for Transceiver<V, F> {
    fn from(other: &Transceiver<U, F>) -> Self {
        let (tx, rx) = flume::unbounded();
        Self {
            crypto: other.crypto.clone(),
            server_address: other.server_address.clone(),
            endpoint: other.endpoint.clone(),
            tx: Some(tx),
            rx: Some(rx),
            function: PhantomData,
            state: PhantomData,
        }
    }
}

impl<U: UniqueIdentifier, F: RxOrTx> Update for Transceiver<U, F, On> {}

impl<U: UniqueIdentifier> Read<U> for Transceiver<U, Transmitter, On> {
    fn read(&mut self, data: Data<U>) {
        if let Some(tx) = self.tx.as_ref() {
            let _ = tx.send(data);
        }
    }
}

impl<U: UniqueIdentifier> Write<U> for Transceiver<U, Receiver, On> {
    fn write(&mut self) -> Option<Data<U>> {
        // if let Some(rx) = self.rx.as_ref() {
        //     if let Ok(data) = rx.recv() {
        //         info!("data forwarded");
        //         Some(data)
        //     } else {
        //         info!("rx failed");
        //         None
        //     }
        // } else {
        //     info!("no rx");
        //     None
        // }
        self.rx.as_ref().and_then(|rx| rx.recv().ok())
    }
}
