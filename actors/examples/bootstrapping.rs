use gmt_dos_actors::prelude::*;
use gmt_dos_actors_dsl::actorscript;
use gmt_dos_clients::{Average, Logging, Sampler, Signals};

#[derive(interface::UID)]
pub enum RRR {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_step = 10;
    let ramp: Signals =
        Signals::new(1, n_step).channels(gmt_dos_clients::Signal::Ramp { a: 1f64, b: 0f64 });
    let logging1 = Logging::new(1);
    let logging2 = Logging::new(1);
    let logging3 = Logging::<f64>::new(1);
    let logging4 = Logging::<f64>::new(1);

    let sampler = Sampler::<Vec<f64>, RRR>::new(vec![0.5]);

    let average = Average::<f64, RRR>::new(1);

    actorscript! {
        #[model(state = completed)]
        1: ramp("Ramp")[RRR] -> &logging1("L1")
        3: ramp("Ramp")[RRR] -> &logging2("L2")
        1: ramp("Ramp")[RRR] -> sampler
        3:  sampler[RRR]! -> &logging3("L3")
        1: ramp("Ramp")[RRR] -> average
        3: average[RRR] -> &logging4("L4")

    }

    println!(
        r"
Full sampling:
{:?}
Downsampling:
{:?}
Downsampling & bootstrapping:
{:?}
Downsampling & averaging:
{:?}
",
        logging1.lock().await.chunks().flatten().collect::<Vec<_>>(),
        logging2.lock().await.chunks().flatten().collect::<Vec<_>>(),
        logging3.lock().await.chunks().flatten().collect::<Vec<_>>(),
        logging4.lock().await.chunks().flatten().collect::<Vec<_>>()
    );

    Ok(())
}
