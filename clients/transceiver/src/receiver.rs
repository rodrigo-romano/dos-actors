use std::{fs, marker::PhantomData, net::SocketAddr};

use gmt_dos_clients::interface::UniqueIdentifier;
use quinn::{ClientConfig, Endpoint};
use tokio::task::JoinHandle;
use tracing::info;

use crate::{Receiver, Transceiver};

impl<U: UniqueIdentifier> Transceiver<U> {
    pub fn receiver() -> ReceiverBuilder<U> {
        ReceiverBuilder {
            client_address: None,
            uid: PhantomData,
        }
    }
}
impl<U: UniqueIdentifier + 'static> Transceiver<U, Receiver> {
    pub fn run<S: Into<String>>(&mut self, address: S, server_name: S) -> JoinHandle<()>
    where
        <U as UniqueIdentifier>::DataType: Send + Sync + for<'a> serde::Deserialize<'a>,
    {
        let endpoint = self.endpoint.clone();
        let tx = self.tx.take().unwrap();
        let address: String = address.into();
        let address: SocketAddr = address.parse().unwrap();
        let server_name: String = server_name.into();
        let handle = tokio::spawn(async move {
            while let Ok(stream) = endpoint.connect(address, &server_name) {
                let Ok( connection) = stream.await else {break};
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
                info!("connection severed");
            }
            info!("disconnecting receiver");
            drop(tx);
        });
        handle
    }
}

pub struct ReceiverBuilder<U: UniqueIdentifier> {
    client_address: Option<String>,
    uid: PhantomData<U>,
}
impl<U: UniqueIdentifier> ReceiverBuilder<U> {
    pub fn client_address<S: Into<String>>(mut self, client_address: S) -> Self {
        self.client_address = Some(client_address.into());
        self
    }
    pub fn build(self) -> crate::Result<Transceiver<U, Receiver>> {
        let dirs = directories_next::ProjectDirs::from("gmt", "dos-clients", "tranceiver").unwrap();
        let path = dirs.data_local_dir();
        let cert_path = path.join("cert.der");

        let mut roots = rustls::RootCertStore::empty();
        let cert = fs::read(cert_path)?;
        roots.add(&rustls::Certificate(cert))?;

        let client_config = ClientConfig::with_root_certificates(roots);
        let address = self
            .client_address
            .unwrap_or("[::]:0".into())
            .parse::<SocketAddr>()?;
        let mut endpoint = Endpoint::client(address)?;
        endpoint.set_default_client_config(client_config);
        Ok(Transceiver::new(endpoint))
    }
}
