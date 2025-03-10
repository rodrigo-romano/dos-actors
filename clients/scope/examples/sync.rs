mod txrx;
use txrx::{Noise, Sin};

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let mut scope = gmt_dos_clients_scope::client::Scope::new()
        .signal::<Sin>()?
        .signal::<Noise>()?;
    let monitor = scope.take_monitor();

    std::thread::spawn(move || {
        rt.block_on(async {
            monitor.join().await?;
            Ok::<(), anyhow::Error>(())
        })
    });

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "GMT DOS Actors Scope",
        native_options,
        Box::new(|cc| {
            scope.run(cc.egui_ctx.clone());
            Ok(Box::new(scope))
        }),
    );

    Ok(())
}
