pub enum Frame {}
impl gmt_dos_clients::interface::UniqueIdentifier for Frame {
    type DataType = Vec<f32>;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    gmt_dos_clients_scope::client::Shot::new("127.0.0.1", "0.0.0.0:0")
        .signal::<Frame>(5001)?
        .show();

    Ok(())
}
