use std::{net::SocketAddr, sync::Arc};

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier};
use gmt_dos_clients_scope::Transceiver;
use quinn::{ClientConfig, Endpoint, ServerConfig};

pub enum TestData {}
impl UniqueIdentifier for TestData {
    type DataType = Vec<f64>;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cert = rcgen::generate_simple_self_signed(vec!["gmt_dos-clients_scope".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key).unwrap();
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = Endpoint::server(
        server_config,
        "127.0.0.1:5001".parse::<SocketAddr>().unwrap(),
    )
    .unwrap();

    let mut certs = rustls::RootCertStore::empty();
    certs.add(&rustls::Certificate(cert_der))?;

    let mut scope = Transceiver::<TestData>::transmitter(certs).build().await?;

    let incoming_conn = endpoint.accept().await.unwrap();
    let conn = incoming_conn.await.unwrap();
    println!(
        "[server] connection accepted: addr={}",
        conn.remote_address()
    );

    // tokio::spawn(async move {
    //     while let Ok((mut send, _)) = scope.connection.open_bi().await {
    //         dbg!("Open uni-connection");
    //         // let data = rx.recv().unwrap();
    //         send.write_all(b"test").await.unwrap();
    //         send.finish().await.unwrap();
    //     }
    // });

    let h = scope.run();

    scope.read(Data::new(vec![1f64, 2f64, 3f64]));

    // if let Ok((_, mut recv)) = conn.accept_bi().await {
    while let Ok(mut recv) = conn.accept_uni().await {
        // Because it is a unidirectional stream, we can only receive not send back.
        println!("{:?}", recv.read_to_end(50).await?);
    }

    // h.await;

    Ok(())
}
