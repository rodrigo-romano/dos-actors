mod crypto;
mod receiver;
mod transmitter;

use std::marker::PhantomData;

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update, Write};
use quinn::Endpoint;

pub use crypto::Crypto;

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
}
pub type Result<T> = std::result::Result<T, TransceiverError>;

/// Receiver function of a [Transceiver]
pub enum Receiver {}
/// Transmitter function of a [Transceiver]
pub enum Transmitter {}
/// [Transceiver] without purpose
pub enum Unset {}
trait RxOrTx {}
impl RxOrTx for Transmitter {}
impl RxOrTx for Receiver {}

#[derive(Debug)]
pub struct Transceiver<U: UniqueIdentifier, F = Unset> {
    crypto: Crypto,
    endpoint: quinn::Endpoint,
    server_address: String,
    tx: Option<flume::Sender<Data<U>>>,
    rx: Option<flume::Receiver<Data<U>>>,
    function: PhantomData<F>,
}
impl<U: UniqueIdentifier, F> Transceiver<U, F> {
    pub fn new<S: Into<String>>(crypto: Crypto, server_address: S, endpoint: Endpoint) -> Self {
        let (tx, rx) = flume::bounded(0);
        Self {
            crypto,
            server_address: server_address.into(),
            endpoint,
            tx: Some(tx),
            rx: Some(rx),
            function: PhantomData,
        }
    }
}

impl<U: UniqueIdentifier, F: RxOrTx> Update for Transceiver<U, F> {}

impl<U: UniqueIdentifier> Read<U> for Transceiver<U, Transmitter> {
    fn read(&mut self, data: Data<U>) {
        if let Some(tx) = self.tx.as_ref() {
            let _ = tx.send(data);
        }
    }
}

impl<U: UniqueIdentifier> Write<U> for Transceiver<U, Receiver> {
    fn write(&mut self) -> Option<Data<U>> {
        self.rx.as_ref().and_then(|rx| rx.recv().ok())
    }
}
