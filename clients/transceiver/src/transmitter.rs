use std::{marker::PhantomData, net::SocketAddr};

use gmt_dos_clients::interface::UniqueIdentifier;
use quinn::Endpoint;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{Crypto, Transceiver, Transmitter};

impl<U: UniqueIdentifier> Transceiver<U> {
    /// [Transceiver] transmitter functionality
    pub fn transmitter<S: Into<String>>(address: S) -> crate::Result<Transceiver<U, Transmitter>> {
        TransmitterBuilder {
            server_address: address.into(),
            uid: PhantomData,
        }
        .build()
    }
}
impl<U: UniqueIdentifier + 'static> Transceiver<U, Transmitter> {
    /// Send data to the receiver
    ///
    /// Communication with the receiver happens in a separate thread.
    /// The transmitter will hold until the receiver calls in.
    pub fn run(&mut self) -> JoinHandle<()>
    where
        <U as UniqueIdentifier>::DataType: Send + Sync + serde::ser::Serialize,
    {
        let endpoint = self.endpoint.clone();
        let rx = self.rx.take().unwrap();
        let handle = tokio::spawn(async move {
            info!("waiting for receiver to connect");
            while let Some(stream) = endpoint.accept().await {
                let Ok( connection) = stream.await else {break};
                info!("transmitter loop");
                loop {
                    let Ok(mut send) = connection.open_uni().await else {break};
                    // let Ok( (mut send,_)) = connection.open_bi().await else {break};
                    info!("outgoing connection");
                    let data = rx.recv().unwrap();
                    let bytes =
                        bincode::serde::encode_to_vec(data, bincode::config::standard()).unwrap();
                    send.write_all(&bytes).await.unwrap();
                    send.finish().await.unwrap();
                }
                info!("connection with receiver lost");
            }
            info!("disconnecting transmitter");
        });
        handle
    }
}

#[derive(Debug)]

struct TransmitterBuilder<U: UniqueIdentifier> {
    server_address: String,
    uid: PhantomData<U>,
}
impl<U: UniqueIdentifier> TransmitterBuilder<U> {
    pub fn build(self) -> crate::Result<Transceiver<U, Transmitter>> {
        let crypto = Crypto::default();
        let server_config = crypto.server()?;
        let address = self.server_address.parse::<SocketAddr>()?;
        let endpoint = Endpoint::server(server_config, address).unwrap();
        Ok(Transceiver::new(crypto, self.server_address, endpoint))
    }
}
