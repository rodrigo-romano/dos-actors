use dos_actors::{
    clients::{windloads, Logging, Sampler},
    prelude::*,
};
use parse_monitors::cfd;
use std::{ops::Deref, time::Instant};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000f64;
    let sim_duration = 400;
    const CFD_RATE: usize = 50;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE as f64;

    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
    let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

    let mut cfd_loads = windloads::CfdLoads::builder(cfd_path.to_str().unwrap())
        .duration(sim_duration)
        .keys(windloads::WindLoads::MirrorCovers.keys())
        .build()
        .unwrap();

    let (mut cfd_source, mut sampler, mut sink) =
        stage!(Vec<f64>: (source[CFD_RATE] => sampler), << sink);

    channel!(cfd_source => sampler => sink);

    let mut logging = Logging::default();
    println!("Starting the model");
    let now = Instant::now();
    spawn!((cfd_source, cfd_loads,), (sampler, Sampler::default(),));
    run!(sink, logging);
    println!("Model run in {}ms", now.elapsed().as_millis());

    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .map(|x| {
                x.chunks(3)
                    .map(|x| x.iter().map(|x| x * x).sum::<f64>().sqrt())
                    .step_by(2)
                    .take(6)
                    .collect::<Vec<f64>>()
            })
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x)),
        complot::complot!(
            "windloads.png",
            xlabel = "Time [s]",
            ylabel = "Force Mag. [N]"
        ),
    )
        .into();

    Ok(())
}
