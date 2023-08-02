mod txrx;
use gmt_dos_clients_scope::Scope;
use txrx::{Noise, Sin};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    Scope::new("127.0.0.1:5001", "127.0.0.1:5000")
        .signal::<Sin>(1e-3)?
        .signal::<Noise>(1e-1)?
        .show();

    Ok(())
}
