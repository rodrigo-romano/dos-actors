mod txrx;
use txrx::{Noise, Sin};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    gmt_dos_clients_scope::Scope::new("127.0.0.1", "127.0.0.1:0")
        .signal::<Sin>(5001)?
        .signal::<Noise>(5002)?
        .show();

    Ok(())
}
