use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{
    interface::{Data, Write},
    Signal, Signals,
};
use gmt_dos_clients_transceiver::{Monitor, Transceiver};

mod txrx;
use txrx::{Noise, Sin, VNoise, VSin};

impl Write<Sin> for Signals {
    fn write(&mut self) -> Option<Data<Sin>> {
        if let Some(data) = <Signals as Write<VSin>>::write(self) {
            Some(Data::new(Vec::from(data)[0]))
        } else {
            None
        }
    }
}

impl Write<Noise> for Signals {
    fn write(&mut self) -> Option<Data<Noise>> {
        if let Some(data) = <Signals as Write<VNoise>>::write(self) {
            Some(Data::new(Vec::from(data)[0]))
        } else {
            None
        }
    }
}

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
        gmt_dos_clients::Signal::Sinusoid {
            amplitude: 1f64,
            sampling_frequency_hz: 100f64,
            frequency_hz: 1f64,
            phase_s: 0f64,
        } + gmt_dos_clients::Signal::Sinusoid {
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
    let sin_tx = Transceiver::<Sin>::transmitter("127.0.0.1:5001")?;
    let noise_tx = Transceiver::<Noise>::transmitter("127.0.0.1:5002")?;

    let mut sin_atx: Terminator<_> = sin_tx.run(&mut monitor).into();
    let mut noise_atx: Terminator<_> = noise_tx.run(&mut monitor).into();

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
