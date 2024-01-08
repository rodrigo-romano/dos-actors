use gmt_dos_actors::{actorscript, prelude::*, system::Sys};
use gmt_dos_clients::{Signal, Signals};
use tweeter::ResHiFi;
use woofer::{AddLoFi, AddResLoFi};

mod tweeter;
mod woofer;
// use crate::sys::*;

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

    let woofer = Sys::new(woofer::Woofer::new()).build()?;
    let tweeter = Sys::new(tweeter::Tweeter::new()).build()?;

    actorscript! {
        #[model(state = running)]
        1: lofi[AddLoFi]~ -> {woofer}[AddResLoFi] -> {tweeter}[ResHiFi]~
    }

    Ok(())
}
