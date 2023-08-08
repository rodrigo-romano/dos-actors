use std::{any::type_name, marker::PhantomData, net::SocketAddr};

use gmt_dos_clients::interface::{Data, UniqueIdentifier};
use quinn::Endpoint;
use tracing::{debug, error, info};

use crate::{Crypto, Monitor, On, Receiver, Transceiver, TransceiverError};

impl<U: UniqueIdentifier> Transceiver<U> {
    /// [Transceiver] receiver functionality
    ///
    /// A receiver is build from both the transmitter and the receiver internet socket addresses
    ///
    /// # Examples
    ///
    /// ```
    /// let tx_address = "127.0.0.1:5001";
    /// let rx_address = "127.0.0.1:500";
    /// let tx = Transceiver::<IO>::receiver(tx_address,rx_address).unwrap();
    /// ```
    pub fn receiver<S: Into<String>, C: Into<String>>(
        server_address: S,
        client_address: C,
    ) -> crate::Result<Transceiver<U, Receiver>> {
        ReceiverBuilder {
            server_address: server_address.into(),
            client_address: client_address.into(),
            uid: PhantomData,
        }
        .build()
    }
}
impl<U: UniqueIdentifier> Transceiver<U, Receiver> {
    /// Spawn a new [Transceiver] receiver
    ///
    /// clone the receiver endpoint that will
    /// `server_address` to connect to
    pub fn spawn<V: UniqueIdentifier, A: Into<String>>(
        &self,
        server_address: A,
    ) -> crate::Result<Transceiver<V, Receiver>> {
        let Self {
            endpoint, crypto, ..
        } = &self;
        let (tx, rx) = flume::unbounded();
        Ok(Transceiver::<V, Receiver> {
            crypto: crypto.clone(),
            endpoint: endpoint.clone(),
            server_address: server_address.into(),
            tx: Some(tx),
            rx: Some(rx),
            function: PhantomData,
            state: PhantomData,
        })
    }
}
impl<U: UniqueIdentifier + 'static> Transceiver<U, Receiver> {
    /// Receive data from the transmitter
    ///
    /// Communication with the transmitter happens in a separate thread.
    /// The receiver will timed-out after 10s if no connection can be established
    /// with the transmitter
    pub fn run(self, monitor: &mut Monitor) -> Transceiver<U, Receiver, On>
    where
        <U as UniqueIdentifier>::DataType: Send + Sync + for<'a> serde::Deserialize<'a>,
    {
        let Self {
            crypto,
            mut endpoint,
            server_address,
            mut tx,
            rx,
            function,
            ..
        } = self;
        let endpoint = endpoint.take().unwrap();
        let tx = tx.take().unwrap();
        let address: SocketAddr = server_address.parse().unwrap();
        let server_name: String = crypto.name.clone();
        let name = type_name::<U>().rsplit("::").next().unwrap();
        let handle = tokio::spawn(async move {
            let stream = endpoint.connect(address, &server_name)?;
            let connection = stream.await.map_err(|e| {
                println!("{name} receiver connection: {e}");
                e
            })?;
            info!(
                "<{}>: incoming connection: {}",
                name,
                connection.remote_address()
            );
            loop {
                match connection.accept_uni().await {
                    Ok(mut recv) => {
                        // receiving data from transmitter
                        let bytes = recv.read_to_end(1_000_000).await?;
                        // debug!("{} bytes received", bytes.len());
                        // decoding data
                        match bincode::serde::decode_from_slice::<(String, Option<Data<U>>), _>(
                            bytes.as_slice(),
                            bincode::config::standard(),
                        ) {
                            // received some data from transmitter and sending to client
                            Ok(((tag, Some(data)), _)) if tag.as_str() == name => {
                                // debug!(" forwarding data");
                                let _ = tx.send(data);
                            }
                            // received none and closing receiver
                            Ok(((tag, None), _)) if tag.as_str() == name => {
                                debug!("<{name}>: data stream ended");
                                break Err(TransceiverError::StreamEnd);
                            }
                            Ok(((tag, _), _)) => {
                                error!("<{name}>: expected {name}, received {tag}");
                                break Err(TransceiverError::DataMismatch(name.into(), tag));
                            }
                            // decoding failure
                            Err(e) => {
                                error!("<{name}>: deserializing failed");
                                break Err(TransceiverError::Decode(e.to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        error!("<{name}>: connection with {address} lost");
                        break Err(TransceiverError::ConnectionError(e));
                    }
                }
            }
            .or_else(|e| {
                info!("<{}>: disconnected ({})", name, e);
                drop(tx);
                match e {
                    TransceiverError::StreamEnd => Ok(()),
                    _ => Err(e),
                }
            })
        });

        monitor.push(handle);
        Transceiver::<U, Receiver, On> {
            crypto,
            endpoint: None,
            server_address,
            tx: None,
            rx,
            function,
            state: PhantomData,
        }
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

pub struct CompactRecvr {
    crypto: Crypto,
    endpoint: Option<quinn::Endpoint>,
}
impl<U: UniqueIdentifier> From<&Transceiver<U, Receiver>> for CompactRecvr {
    fn from(value: &Transceiver<U, Receiver>) -> Self {
        let Transceiver::<U, Receiver> {
            crypto, endpoint, ..
        } = value;
        Self {
            crypto: crypto.clone(),
            endpoint: endpoint.clone(),
        }
    }
}

impl CompactRecvr {
    /// Spawn a new [Transceiver] receiver
    ///
    /// clone the receiver endpoint that will
    /// `server_address` to connect to
    pub fn spawn<V: UniqueIdentifier, A: Into<String>>(
        &self,
        server_address: A,
    ) -> crate::Result<Transceiver<V, Receiver>> {
        let Self {
            endpoint, crypto, ..
        } = &self;
        let (tx, rx) = flume::unbounded();
        Ok(Transceiver::<V, Receiver> {
            crypto: crypto.clone(),
            endpoint: endpoint.clone(),
            server_address: server_address.into(),
            tx: Some(tx),
            rx: Some(rx),
            function: PhantomData,
            state: PhantomData,
        })
    }
}
