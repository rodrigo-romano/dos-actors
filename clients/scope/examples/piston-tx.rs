use gmt_dos_actors::actorscript;
use gmt_dos_clients::{Signal, Signals};
use gmt_dos_clients_io::optics::WfeRms;
use gmt_dos_clients_scope::server::{Monitor, Scope};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sampling_frequency = 1000_usize;

    let piston: Signals = Signals::new(1, 3).channel(0, Signal::Constant(25f64));
    // .channel(1, Signal::Constant(50f64));
    // .channel(2, Signal::Constant(75f64));

    let mut monitor = Monitor::new();
    let scope = Scope::<WfeRms>::builder("127.0.0.1:5001", &mut monitor)
        .sampling_period((sampling_frequency as f64).recip())
        .build()?;

    actorscript!(
        1: piston[WfeRms] -> scope
    );

    monitor.await?;

    Ok(())
}
