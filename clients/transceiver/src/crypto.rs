use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Result;
use quinn::{ClientConfig, ServerConfig};
use tracing::info;

/// Transceiver encryption settings
///
/// The settings for the communication encryption consists in:
///  * the certificate file name: `gmt_dos-clients_transceiver_cert.der`
///  * the private key file name: `gmt_dos-clients_transceiver_key.der`
///  * the server name: `gmt_dos-clients_transceiver`
#[derive(Debug, Clone)]
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
    /// Generates the certificate and the private key
    ///
    /// The cerficate and the private key are written to the specified files
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
    /// Returns [quinn](https://docs.rs/quinn/latest/quinn/crypto/trait.ServerConfig.html) server configuration
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
    /// Returns [quinn](https://docs.rs/quinn/latest/quinn/struct.ClientConfig.html#) client configuration
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
