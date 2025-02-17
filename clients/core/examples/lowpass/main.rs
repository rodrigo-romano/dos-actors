use gmt_dos_actors::actorscript;
use gmt_dos_clients::{
    low_pass_filter::LowPassFilter,
    signals::{Signal, Signals},
};
use interface::UID;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_step = 10_000;
    let signal: Signals = Signals::new(1, n_step).channel(
        0,
        Signal::Sinusoid {
            amplitude: 1.,
            sampling_frequency_hz: 1000.,
            frequency_hz: 0.5,
            phase_s: 0.,
        } + Signal::Sinusoid {
            amplitude: 0.1,
            sampling_frequency_hz: 1000.,
            frequency_hz: 10.,
            phase_s: 0.2,
        },
    );
    let lpf = LowPassFilter::new(1, 0.02);

    actorscript!(
        1: signal[SinSin]~ -> lpf[LpfSinSin]~
    );

    Ok(())
}

#[derive(UID)]
#[uid(port = 5001)]
pub enum SinSin {}

#[derive(UID)]
#[uid(port = 5002)]
pub enum LpfSinSin {}
