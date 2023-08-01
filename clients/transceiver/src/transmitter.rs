use std::{fmt::Debug, marker::PhantomData, net::SocketAddr};

use bincode::{config, serde::encode_to_vec};
use gmt_dos_clients::interface::{Data, UniqueIdentifier};
use quinn::Endpoint;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{Crypto, Monitor, On, Transceiver, TransceiverError, Transmitter};

impl<U: UniqueIdentifier> Transceiver<U> {
    /// [Transceiver] transmitter functionality
    ///
    /// A transmitter is build from its internet socket address
    ///
    /// # Examples
    ///
    /// ```
    /// let address = "127.0.0.1:5001";
    /// let tx = Transceiver::<IO>::transmitter(address).unwrap();
    /// ```
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
    pub fn run(self, monitor: &mut Monitor) -> Transceiver<U, Transmitter, On>
    where
        <U as UniqueIdentifier>::DataType: Send + Sync + serde::ser::Serialize,
    {
        let Self {
            crypto,
            mut endpoint,
            server_address,
            tx,
            mut rx,
            function,
            ..
        } = self;
        let endpoint = endpoint.take().unwrap();
        let rx = rx.take().unwrap();
        let handle: JoinHandle<Result<(), TransceiverError>> = tokio::spawn(async move {
            info!("waiting for receiver to connect");
            'endpoint: {
                while let Some(stream) = endpoint.accept().await {
                    let connection = stream.await?;
                    info!("transmitter loop");
                    while let Ok(mut send) = connection.open_uni().await {
                        info!("outgoing connection");
                        // check if client sent data
                        match rx.recv() {
                            // received some data from client, encoding and sending some to receiver
                            Ok(data) => match encode_to_vec(Some(data), config::standard()) {
                                Ok(bytes) => {
                                    send.write_all(&bytes).await?;
                                    send.finish().await?;
                                }
                                Err(e) => {
                                    break 'endpoint Err(TransceiverError::Encode(e.to_string()))
                                }
                            },
                            // received none, sending none to receiver and closing transmitter
                            Err(flume::RecvError::Disconnected) => {
                                info!("rx disconnected");
                                let bytes: Vec<u8> =
                                    encode_to_vec(Option::<Data<U>>::None, config::standard())
                                        .map_err(|e| TransceiverError::Encode(e.to_string()))?;
                                send.write_all(&bytes).await?;
                                send.finish().await?;
                                break 'endpoint Ok(());
                            }
                        }
                    }
                    info!("connection with receiver lost");
                }
                Ok(())
            }?;
            info!("disconnecting transmitter");
            Ok(())
        });
        monitor.push(handle);
        Transceiver::<U, Transmitter, On> {
            crypto,
            endpoint: None,
            server_address,
            tx,
            rx: None,
            function,
            state: PhantomData,
        }
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
