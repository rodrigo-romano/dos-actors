use gmt_dos_actors::prelude::*;
use gmt_dos_clients::Signals;
use gmt_dos_clients_transceiver::Transceiver;

mod txrx;
use txrx::Sin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let signal: Signals = Signals::new(1, 10).channels(gmt_dos_clients::Signal::Sinusoid {
        amplitude: 1f64,
        sampling_frequency_hz: 4f64,
        frequency_hz: 1f64,
        phase_s: 0f64,
    });
    let mut signal: Initiator<_> = signal.into();
    let mut tx = Transceiver::<Sin>::transmitter("127.0.0.1:5001")?;
    tx.run();
    let mut atx: Terminator<_> = tx.into();

    signal.add_output().build::<Sin>().into_input(&mut atx)?;

    model!(signal, atx)
        .name("tx")
        .flowchart()
        .check()?
        .run()
        .await?;

    Ok(())
}
