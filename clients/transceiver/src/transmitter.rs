use std::{fs, io, marker::PhantomData, net::SocketAddr};

use gmt_dos_clients::interface::UniqueIdentifier;
use quinn::{Endpoint, ServerConfig};
use tokio::task::JoinHandle;
use tracing::info;

use crate::{Transceiver, Transmitter};

impl<U: UniqueIdentifier> Transceiver<U> {
    pub fn transmitter() -> TransmitterBuilder<U> {
        TransmitterBuilder {
            server_address: None,
            uid: PhantomData,
        }
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

pub struct TransmitterBuilder<U: UniqueIdentifier> {
    server_address: Option<String>,
    uid: PhantomData<U>,
}
impl<U: UniqueIdentifier> TransmitterBuilder<U> {
    pub fn server_address<S: Into<String>>(mut self, server_address: S) -> Self {
        self.server_address = Some(server_address.into());
        self
    }
    pub fn build(self) -> crate::Result<Transceiver<U, Transmitter>> {
        let dirs = directories_next::ProjectDirs::from("gmt", "dos-clients", "tranceiver").unwrap();
        let path = dirs.data_local_dir();
        let cert_path = path.join("cert.der");
        let key_path = path.join("key.der");
        let (cert, key) = match fs::read(&cert_path).and_then(|x| Ok((x, fs::read(&key_path)?))) {
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                info!("generating self-signed certificate");
                let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
                let key = cert.serialize_private_key_der();
                let cert = cert.serialize_der().unwrap();
                fs::create_dir_all(path)?;
                fs::write(&cert_path, &cert)?;
                fs::write(&key_path, &key)?;
                Ok((cert, key))
            }
            value => value,
        }?;

        let key = rustls::PrivateKey(key);
        let cert = rustls::Certificate(cert);

        let server_config = ServerConfig::with_single_cert(vec![cert], key)?;

        let address = self
            .server_address
            .unwrap_or("127.0.0.1:5001".into())
            .parse::<SocketAddr>()?;
        let endpoint = Endpoint::server(server_config, address).unwrap();
        Ok(Transceiver::new(endpoint))
    }
}
