use std::{any::type_name, marker::PhantomData, net::SocketAddr};

use gmt_dos_clients::interface::{Data, UniqueIdentifier};
use quinn::Endpoint;
use tracing::debug;

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
impl<U: UniqueIdentifier> Transceiver<U, Receiver> {
    /// Spawn a new [Transceiver] receiver
    ///
    /// a new receiver endpoint is generated if a client address is given
    /// otherwise the receiver endpoint is cloned
    pub fn spawn<V: UniqueIdentifier, A: Into<String>>(
        &self,
        client_address: Option<A>,
    ) -> crate::Result<Transceiver<V, Receiver>> {
        let Self {
            endpoint,
            crypto,
            server_address,
            ..
        } = &self;
        let endpoint = if let Some(client_address) = client_address {
            let address = client_address.into().parse::<SocketAddr>()?;
            let mut endpoint = Endpoint::client(address)?;
            endpoint.set_default_client_config(crypto.client()?);
            Some(endpoint)
        } else {
            endpoint.clone()
        };
        let (tx, rx) = flume::unbounded();
        Ok(Transceiver::<V, Receiver> {
            crypto: crypto.clone(),
            endpoint,
            server_address: server_address.clone(),
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
        let name = type_name::<U>();
        let handle = tokio::spawn(async move {
            debug!("trying to connect to the transmitter");
            'endpoint: {
                while let Ok(stream) = endpoint.connect(address, &server_name) {
                    let connection = stream.await.map_err(|e| {
                        println!("{name} receiver connection: {e}");
                        e
                    })?;
                    debug!("receiver loop");
                    while let Ok(mut recv) = connection.accept_uni().await {
                        debug!("incoming connection");
                        // receiving data from transmitter
                        let bytes = recv.read_to_end(1_000_000).await?;
                        // debug!("{} bytes received", bytes.len());
                        // decoding data
                        match bincode::serde::decode_from_slice::<Option<Data<U>>, _>(
                            bytes.as_slice(),
                            bincode::config::standard(),
                        ) {
                            // received some data from transmitter and sending to client
                            Ok((Some(data), _)) => {
                                // debug!(" forwarding data");
                                let _ = tx.send(data);
                            }
                            // received none and closing receiver
                            Ok((None, _)) => {
                                debug!("data stream ended");
                                break 'endpoint Ok(());
                            }
                            // decoding failure
                            Err(e) => {
                                // debug!("deserializing failed");
                                break 'endpoint Err(TransceiverError::Decode(e.to_string()));
                            }
                        }
                    }
                    debug!("connection with transmitter lost");
                }
                Ok(())
            }?;
            debug!("disconnecting receiver");
            drop(tx);
            Ok(())
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
    server_address: String,
}
impl<U: UniqueIdentifier> From<&Transceiver<U, Receiver>> for CompactRecvr {
    fn from(value: &Transceiver<U, Receiver>) -> Self {
        let Transceiver::<U, Receiver> {
            crypto,
            endpoint,
            server_address,
            ..
        } = value;
        Self {
            crypto: crypto.clone(),
            endpoint: endpoint.clone(),
            server_address: server_address.clone(),
        }
    }
}
impl<U: UniqueIdentifier> From<&CompactRecvr> for Transceiver<U, Receiver> {
    fn from(value: &CompactRecvr) -> Self {
        let CompactRecvr {
            crypto,
            endpoint,
            server_address,
        } = value;

        let (tx, rx) = flume::unbounded();
        Transceiver::<U, Receiver> {
            crypto: crypto.clone(),
            endpoint: endpoint.clone(),
            server_address: server_address.clone(),
            tx: Some(tx),
            rx: Some(rx),
            function: PhantomData,
            state: PhantomData,
        }
    }
}
