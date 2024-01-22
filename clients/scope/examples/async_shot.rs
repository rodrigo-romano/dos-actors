pub enum Wave {}
impl interface::UniqueIdentifier for Wave {
    type DataType = Vec<f64>;
    const PORT: u16 = 5001;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    gmt_dos_clients_scope::client::Shot::new()
        .signal::<Wave>()?
        .show();

    Ok(())
}
