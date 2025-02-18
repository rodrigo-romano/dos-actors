use gmt_dos_actors::prelude::*;
use gmt_dos_clients::signals::{Signal, Signals};
use gmt_dos_clients_scope::server::{Monitor, Scope};

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

    let n_step = 10_000;
    let sin: Signals = Signals::new(1, n_step).channels(
        Signal::Sinusoid {
            amplitude: 1f64,
            sampling_frequency_hz: 100f64,
            frequency_hz: 1f64,
            phase_s: 0f64,
        } + Signal::Sinusoid {
            amplitude: 5f64,
            sampling_frequency_hz: 100f64,
            frequency_hz: 0.1f64,
            phase_s: 0f64,
        },
    );
    let mut sin: Initiator<_> = sin.into();

    let noise: Signals = Signals::new(1, n_step / 100)
        .channels(Signal::WhiteNoise(rand_distr::Normal::new(0f64, 1f64)?));
    let mut noise: Initiator<_> = noise.into();

    let mut monitor = Monitor::new();

    let mut sin_atx: Terminator<_> = Scope::<Sin>::builder(&mut monitor)
        .sampling_period(1e-3)
        .build()?
        .into(); //sin_tx.run(&mut monitor).into();
    let mut noise_atx: Terminator<_> = Scope::<Noise>::builder(&mut monitor)
        .sampling_period(1e-1)
        .build()?
        .into();

    sin.add_output()
        .unbounded()
        .build::<Sin>()
        .into_input(&mut sin_atx)?;
    noise
        .add_output()
        .unbounded()
        .build::<Noise>()
        .into_input(&mut noise_atx)?;

    model!(sin, sin_atx, noise, noise_atx)
        .name("tx")
        .flowchart()
        .check()?
        .run()
        .await?;

    monitor.await?;

    Ok(())
}
