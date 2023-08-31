use std::{f64::consts::PI, thread, time};

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{
    interface::{Data, Read, UniqueIdentifier, Update, Write},
    Signals,
};
use gmt_dos_clients_scope::server::Shot;
use gmt_dos_clients_transceiver::Monitor;

pub struct SinSin {
    size: [usize; 2],
    period: f64,
}
impl SinSin {
    pub fn new(size: [usize; 2], period: f64) -> Self {
        Self { size, period }
    }
    pub fn wave(&self) -> Vec<f64> {
        let [n, m] = self.size;
        let mut w = Vec::<f64>::with_capacity(n * m);
        let n = n as i32;
        let m = m as i32;
        for i in 0..n {
            let x = 2f64 * PI * (i - n / 2) as f64 / n as f64 / self.period;
            for j in 0..m {
                let y = 2f64 * PI * (j - m / 2) as f64 / m as f64 / self.period;
                w.push(x.sin() * y.sin());
            }
        }
        w
    }
}

impl Update for SinSin {}

pub enum Wave {}
impl UniqueIdentifier for Wave {
    type DataType = Vec<f64>;
}

impl Write<Wave> for SinSin {
    fn write(&mut self) -> Option<Data<Wave>> {
        thread::sleep(time::Duration::from_millis(50));
        Some(Data::new(self.wave()))
    }
}

pub enum Period {}
impl UniqueIdentifier for Period {
    type DataType = Vec<f64>;
}

impl Read<Period> for SinSin {
    fn read(&mut self, data: Data<Period>) {
        self.period = 1. / *&data[0].abs();
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

    let n_step = 250;
    let period: Signals = Signals::new(1, n_step).channels(gmt_dos_clients::Signal::Sinusoid {
        amplitude: 8f64,
        sampling_frequency_hz: 100f64,
        frequency_hz: 1f64,
        phase_s: 0f64,
    });
    let mut period: Initiator<_> = period.into();

    let n = 64;
    let size = [n, n];
    let mut wave: Actor<_> = SinSin::new(size, 4f64).into();

    let mut monitor: Monitor = Monitor::new();

    let mut atx: Terminator<_> = Shot::<Wave>::builder("127.0.0.1:5001", &mut monitor, size)
        .minmax((-1f64, 1f64))
        .build()?
        .into();

    period
        .add_output()
        .build::<Period>()
        .into_input(&mut wave)?;

    wave.add_output()
        // .unbounded()
        .build::<Wave>()
        .into_input(&mut atx)?;

    model!(period, wave, atx)
        .name("wave")
        .flowchart()
        .check()?
        .run()
        .await?;

    monitor.await?;

    Ok(())
}
