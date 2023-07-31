use gmt_dos_actors::prelude::*;
use gmt_dos_clients_transceiver::Transceiver;

mod txrx;
use txrx::{Print, Sin};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let mut rx = Transceiver::<Sin>::receiver().build()?;
    rx.run("127.0.0.1:5001", "localhost");
    let mut arx: Initiator<_> = rx.into();
    let mut rx_print: Terminator<_> = Print.into();

    arx.add_output().build::<Sin>().into_input(&mut rx_print)?;

    model!(arx, rx_print)
        .name("rx")
        .flowchart()
        .check()?
        .run()
        .await?;

    Ok(())
}
