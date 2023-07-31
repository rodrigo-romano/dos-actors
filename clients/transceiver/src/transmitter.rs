use std::{marker::PhantomData, net::SocketAddr};

use gmt_dos_clients::interface::UniqueIdentifier;
use quinn::Endpoint;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{Crypto, Transceiver, Transmitter};

impl<U: UniqueIdentifier> Transceiver<U> {
    pub fn transmitter<S: Into<String>>(address: S) -> crate::Result<Transceiver<U, Transmitter>> {
        TransmitterBuilder {
            server_address: address.into(),
            uid: PhantomData,
        }
        .build()
    }
}
impl<U: UniqueIdentifier + 'static> Transceiver<U, Transmitter> {
    pub fn run(&mut self) -> JoinHandle<()>
    where
        <U as UniqueIdentifier>::DataType: Send + Sync + serde::ser::Serialize,
    {
        let endpoint = self.endpoint.clone();
        let rx = self.rx.take().unwrap();
        let handle = tokio::spawn(async move {
            // while let Ok((mut send, _)) = connection.open_bi().await {
            while let Some(stream) = endpoint.accept().await {
                let Ok( connection) = stream.await else {break};
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
            }
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
