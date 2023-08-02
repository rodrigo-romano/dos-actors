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

    let scope = Scope::new("127.0.0.1:5001", "127.0.0.1:5000")
        .signal::<Sin>()?
        .signal::<Noise>()?
        .run();

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "GMT DOS Actors Scope",
        native_options,
        Box::new(|_cc| Box::new(scope)),
    );

    Ok(())
}
