use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Result;
use quinn::{ClientConfig, ServerConfig};
use tracing::info;

#[derive(Debug)]
pub struct Crypto {
    cert_path: PathBuf,
    cert_file: String,
    key_file: String,
    pub(crate) name: String,
}
impl Default for Crypto {
    fn default() -> Self {
        Self {
            cert_path: Path::new(".").into(),
            cert_file: "gmt_dos-clients_transceiver_cert.der".to_string(),
            key_file: "gmt_dos-clients_transceiver_key.der".to_string(),
            name: "gmt_dos-clients_transceiver".into(),
        }
    }
}
impl Crypto {
    pub fn generate(&self) -> Result<()> {
        info!("generating self-signed certificate");
        let Crypto {
            cert_path,
            cert_file,
            key_file,
            name,
        } = self;
        let cert: rcgen::Certificate =
            rcgen::generate_simple_self_signed(vec![name.into()]).unwrap();
        let key = cert.serialize_private_key_der();
        let cert = cert.serialize_der().unwrap();
        fs::create_dir_all(cert_path)?;
        fs::write(cert_path.join(cert_file), &cert)?;
        fs::write(cert_path.join(key_file), &key)?;
        Ok(())
    }
    pub fn server(&self) -> Result<ServerConfig> {
        let Crypto {
            cert_path,
            cert_file,
            key_file,
            ..
        } = self;
        let cert = fs::read(cert_path.join(cert_file))?;
        let key = fs::read(cert_path.join(key_file))?;
        let key = rustls::PrivateKey(key);
        let cert = rustls::Certificate(cert);

        Ok(ServerConfig::with_single_cert(vec![cert], key)?)
    }
    pub fn client(&self) -> Result<ClientConfig> {
        let Crypto {
            cert_path,
            cert_file,
            ..
        } = self;
        let cert = fs::read(cert_path.join(cert_file))?;
        let mut roots = rustls::RootCertStore::empty();
        roots.add(&rustls::Certificate(cert))?;
        Ok(ClientConfig::with_root_certificates(roots))
    }
}
