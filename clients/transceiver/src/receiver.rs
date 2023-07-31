use std::{marker::PhantomData, net::SocketAddr};

use gmt_dos_clients::interface::UniqueIdentifier;
use quinn::Endpoint;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{Crypto, Receiver, Transceiver};

impl<U: UniqueIdentifier> Transceiver<U> {
    /// [Transceiver] receiver functionality
    pub fn receiver<S: Into<String>>(
        server_address: S,
        client_address: S,
    ) -> crate::Result<Transceiver<U, Receiver>> {
        ReceiverBuilder {
            server_address: server_address.into(),
            client_address: client_address.into(),
            uid: PhantomData,
        }
        .build()
    }
}
impl<U: UniqueIdentifier + 'static> Transceiver<U, Receiver> {
    /// Receive data from the transmitter
    ///
    /// Communication with the transmitter happens in a separate thread.
    /// The receiver will timed-out after 10s if no connection can be established
    /// with the transmitter
    pub fn run(&mut self) -> JoinHandle<()>
    where
        <U as UniqueIdentifier>::DataType: Send + Sync + for<'a> serde::Deserialize<'a>,
    {
        let endpoint = self.endpoint.clone();
        let tx = self.tx.take().unwrap();
        let address: SocketAddr = self.server_address.parse().unwrap();
        let server_name: String = self.crypto.name.clone();
        let handle = tokio::spawn(async move {
            info!("trying to connect to the transmitter");
            while let Ok(stream) = endpoint.connect(address, &server_name) {
                let Ok( connection) = stream.await else {break};
                info!("receiver loop");
                loop {
                    let Ok(mut recv) = connection.accept_uni().await else {break};
                    // let Ok((_,mut recv)) =/ connection.accept_bi().await else {break};
                    info!("incoming connection");
                    let bytes = recv.read_to_end(1_000_000).await.unwrap();
                    let (data, _): (gmt_dos_clients::interface::Data<U>, usize) =
                        bincode::serde::decode_from_slice(
                            bytes.as_slice(),
                            bincode::config::standard(),
                        )
                        .unwrap();
                    let _ = tx.send(data);
                }
                info!("connection with transmitter lost");
            }
            info!("disconnecting receiver");
            drop(tx);
        });
        handle
    }
}

#[derive(Debug)]
struct ReceiverBuilder<U: UniqueIdentifier> {
    server_address: String,
    client_address: String,
    uid: PhantomData<U>,
}
impl<U: UniqueIdentifier> ReceiverBuilder<U> {
    pub fn build(self) -> crate::Result<Transceiver<U, Receiver>> {
        let crypto = Crypto::default();
        let client_config = crypto.client()?;
        let address = self.client_address.parse::<SocketAddr>()?;
        let mut endpoint = Endpoint::client(address)?;
        endpoint.set_default_client_config(client_config);
        Ok(Transceiver::new(crypto, self.server_address, endpoint))
    }
}
