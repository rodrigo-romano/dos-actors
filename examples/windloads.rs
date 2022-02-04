use dos_actors::{
    clients::{windloads, windloads::CS, Logging, Sampler},
    prelude::*,
};
use parse_monitors::cfd;
use std::{ops::Deref, time::Instant};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 20f64;
    let sim_duration = 30;
    const CFD_RATE: usize = 1;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE as f64;

    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
    let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

    let loads = windloads::WindLoads::M1Cell;

    let mut fem = fem::FEM::from_env()?;
    println!("{}", fem);
    fem.keep_inputs(&[0])
        .filter_inputs_by(&[0], |x| {
            windloads::WindLoads::M1Cell
                .fem()
                .iter()
                .fold(false, |b, p| b || x.descriptions.contains(p))
        })
        .keep_outputs(&[100]);
    println!("{}", fem);
    let locations: Vec<CS> = fem.inputs[0]
        .as_ref()
        .unwrap()
        .get_by(|x| Some(CS::OSS(x.properties.location.as_ref().unwrap().clone())))
        .into_iter()
        .step_by(6)
        .collect();

    let mut cfd_loads = windloads::CfdLoads::builder(cfd_path.to_str().unwrap())
        .duration(sim_duration)
        .nodes(loads.keys(), locations)
        .m1_segments()
        .build()
        .unwrap();

    let (mut cfd_source, mut sampler, mut sink) =
        stage!(Vec<f64>: (source[CFD_RATE] => sampler), << sink);

    channel!(cfd_source => sink; 2);

    let mut logging = Logging::default();
    println!("Starting the model");
    let now = Instant::now();
    spawn!((cfd_source, cfd_loads,), (sampler, Sampler::default(),));
    run!(sink, logging);
    println!("{}", logging.len());
    println!("Model run in {}ms", now.elapsed().as_millis());

    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .map(|x| {
                let forces = x.chunks(3).step_by(2).flatten().collect::<Vec<&f64>>();
                let fx = forces.iter().step_by(3).fold(0f64, |a, &&x| a + x);
                let fy = forces.iter().skip(1).step_by(3).fold(0f64, |a, &&x| a + x);
                let fz = forces.iter().skip(2).step_by(3).fold(0f64, |a, &&x| a + x);
                vec![fx, fy, fz]
            })
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x)),
        complot::complot!(
            "examples/figures/windloads.png",
            xlabel = "Time [s]",
            ylabel = "Force Mag. [N]"
        ),
    )
        .into();

    Ok(())
}
