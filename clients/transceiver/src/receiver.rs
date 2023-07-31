use std::{marker::PhantomData, net::SocketAddr};

use gmt_dos_clients::interface::{Data, UniqueIdentifier};
use quinn::Endpoint;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{Crypto, Receiver, Transceiver, TransceiverError};

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
    pub fn run(&mut self) -> JoinHandle<Result<(), TransceiverError>>
    where
        <U as UniqueIdentifier>::DataType: Send + Sync + for<'a> serde::Deserialize<'a>,
    {
        let endpoint = self.endpoint.clone();
        let tx = self.tx.take().unwrap();
        let address: SocketAddr = self.server_address.parse().unwrap();
        let server_name: String = self.crypto.name.clone();
        let handle = tokio::spawn(async move {
            info!("trying to connect to the transmitter");
            'endpoint: {
                while let Ok(stream) = endpoint.connect(address, &server_name) {
                    let connection = stream.await?;
                    info!("receiver loop");
                    while let Ok(mut recv) = connection.accept_uni().await {
                        info!("incoming connection");
                        // receiving data from transmitter
                        let bytes = recv.read_to_end(1_000_000).await?;
                        // info!("{} bytes received", bytes.len());
                        // decoding data
                        match bincode::serde::decode_from_slice::<Option<Data<U>>, _>(
                            bytes.as_slice(),
                            bincode::config::standard(),
                        ) {
                            // received some data from transmitter and sending to client
                            Ok((Some(data), _)) => {
                                // info!(" forwarding data");
                                let _ = tx.send(data);
                            }
                            // received none and closing receiver
                            Ok((None, _)) => {
                                info!("data stream ended");
                                break 'endpoint Ok(());
                            }
                            // decoding failure
                            Err(e) => {
                                // info!("deserializing failed");
                                break 'endpoint Err(TransceiverError::Decode(e.to_string()));
                            }
                        }
                    }
                    info!("connection with transmitter lost");
                }
                Ok(())
            }?;
            info!("disconnecting receiver");
            drop(tx);
            Ok(())
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
