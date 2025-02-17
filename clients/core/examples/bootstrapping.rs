use gmt_dos_actors::actorscript;
use gmt_dos_clients::{
    average::Average,
    logging::Logging,
    sampler::Sampler,
    signals::{Signal, Signals},
};

#[derive(interface::UID)]
pub enum RRR {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let n_step = 10;
    let ramp: Signals = Signals::new(1, n_step).channels(Signal::Ramp { a: 1f64, b: 0f64 });
    let logging1 = Logging::new(1);
    let logging2 = Logging::new(1);
    let logging3 = Logging::<f64>::new(1);
    let logging4 = Logging::<f64>::new(1);

    let sampler = Sampler::<Vec<f64>, RRR>::new(vec![0.5]);

    let average = Average::<f64, RRR>::new(1);

    actorscript! {
        #[model(state = completed)]
        #[labels(ramp="Ramp",logging1="L1",logging2="L2",logging3="L3",logging4="L4")]
        1: ramp[RRR] -> logging1
        3: ramp[RRR] -> logging2
        1: ramp[RRR] -> sampler
        3:  sampler[RRR]! -> logging3
        1: ramp[RRR] -> average
        3: average[RRR] -> logging4

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
