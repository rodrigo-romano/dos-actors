use gmt_dos_actors::prelude::*;
use gmt_dos_clients_transceiver::{Receiver, Transceiver};

mod txrx;
use txrx::{ISin, Print, Sin};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let mut sin_rx = Transceiver::<Sin>::receiver("127.0.0.1:5001", "127.0.0.1:5000")?;
    let mut isin_rx = Transceiver::<ISin, Receiver>::from(&sin_rx);

    sin_rx.run();
    let mut sin_arx: Initiator<_> = sin_rx.into();
    let mut sin_rx_print: Terminator<_> = Print.into();

    isin_rx.run();
    let mut isin_arx: Initiator<_> = isin_rx.into();
    let mut isin_rx_print: Terminator<_> = Print.into();

    sin_arx
        .add_output()
        .unbounded()
        .build::<Sin>()
        .into_input(&mut sin_rx_print)?;

    isin_arx
        .add_output()
        .unbounded()
        .build::<ISin>()
        .into_input(&mut isin_rx_print)?;

    model!(sin_arx, sin_rx_print, isin_arx, isin_rx_print)
        .name("rx")
        .flowchart()
        .check()?
        .run()
        .await?;

    Ok(())
}
