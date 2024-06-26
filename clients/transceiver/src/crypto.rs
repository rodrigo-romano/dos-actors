use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::Result;
use quinn::{ClientConfig, ServerConfig, TransportConfig};
use tracing::info;

const TIME_OUT: u64 = 180;

pub struct CryptoBuilder {
    // cert_path: PathBuf,
    cert_file: String,
    key_file: String,
}
impl Default for CryptoBuilder {
    fn default() -> Self {
        Self {
            cert_file: "gmt_dos-clients_transceiver_cert.der".to_string(),
            key_file: "gmt_dos-clients_transceiver_key.der".to_string(),
        }
    }
}
impl CryptoBuilder {
    pub fn certificate<S: Into<String>>(mut self, cert_file: S) -> Self {
        self.cert_file = cert_file.into();
        self
    }
    pub fn key<S: Into<String>>(mut self, key_file: S) -> Self {
        self.key_file = key_file.into();
        self
    }
    pub fn build(self) -> Crypto {
        Crypto {
            cert_file: self.cert_file,
            key_file: self.key_file,
            ..Default::default()
        }
    }
}

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
    pub fn builder() -> CryptoBuilder {
        Default::default()
    }
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
    #[cfg(not(feature = "insecure"))]
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
    #[cfg(feature = "insecure")]
    pub fn server(&self) -> Result<ServerConfig> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.serialize_der().unwrap();
        let key = cert.serialize_private_key_der();
        let key = rustls::PrivateKey(key);
        let cert_chain = vec![rustls::Certificate(cert_der.clone())];

        Ok(ServerConfig::with_single_cert(cert_chain, key)?)
    }
    /// Returns [quinn](https://docs.rs/quinn/latest/quinn/struct.ClientConfig.html#) client configuration
    #[cfg(not(feature = "insecure"))]
    pub fn client(&self) -> Result<ClientConfig> {
        let Crypto {
            cert_path,
            cert_file,
            ..
        } = self;
        let cert = fs::read(cert_path.join(cert_file))?;
        let mut roots = rustls::RootCertStore::empty();
        roots.add(&rustls::Certificate(cert))?;
        let mut config = TransportConfig::default();
        config.max_idle_timeout(Some(std::time::Duration::from_secs(TIME_OUT).try_into()?));
        let mut client_config = ClientConfig::with_root_certificates(roots);
        client_config.transport_config(std::sync::Arc::new(config));
        Ok(client_config)
    }
    #[cfg(feature = "insecure")]
    pub fn client(&self) -> Result<ClientConfig> {
        let mut config = TransportConfig::default();
        config.max_idle_timeout(Some(std::time::Duration::from_secs(TIME_OUT).try_into()?));

        let crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(insecure::SkipServerVerification::new())
            .with_no_client_auth();
        let mut client_config = ClientConfig::new(std::sync::Arc::new(crypto));
        client_config.transport_config(std::sync::Arc::new(config));
        Ok(client_config)
    }
}

#[cfg(feature = "insecure")]
pub mod insecure {
    use std::sync::Arc;

    pub(super) struct SkipServerVerification;

    impl SkipServerVerification {
        pub(super) fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    impl rustls::client::ServerCertVerifier for SkipServerVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::Certificate,
            _intermediates: &[rustls::Certificate],
            _server_name: &rustls::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> std::result::Result<rustls::client::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::ServerCertVerified::assertion())
        }
    }
}
