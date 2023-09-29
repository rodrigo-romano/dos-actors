use gmt_dos_actors::{actorscript, subsystem::SubSystem};
use gmt_dos_clients::{Signal, Signals};
mod common;
use crate::common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sampling_frequency_hz = 1_000.;
    let lofi: Signals = Signals::new(1, 4_000).channels(
        Signal::Sinusoid {
            amplitude: 1.,
            sampling_frequency_hz,
            frequency_hz: 1.,
            phase_s: 0.,
        } + Signal::Sinusoid {
            amplitude: 0.25,
            sampling_frequency_hz,
            frequency_hz: 10.,
            phase_s: 0.,
        },
    );

    let mut woofer = SubSystem::new(Woofer::new())
        .name("woofer")
        .build()?
        .flowchart();
    let mut tweeter = SubSystem::new(Tweeter::new())
        .name("tweeter")
        .build()?
        .flowchart();

    actorscript! {
        1: lofi[AddLoFi]~ -> {woofer}[ResLoFi]~ -> {tweeter}[ResHiFi]~
    }

    Ok(())
}
