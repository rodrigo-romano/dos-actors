use dos_actors::{
    clients::{windloads, windloads::WindLoads::*, windloads::CS, Logging, Sampler},
    prelude::*,
};
use parse_monitors::cfd;
use std::{ops::Deref, time::Instant};
use welch_sde::{Build, PowerSpectrum, Welch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 20f64;
    let sim_duration = 1f64;
    const CFD_RATE: usize = 1;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE as f64;

    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
    let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

    let loads = vec![
        TopEnd,
        /*M2Baffle,
        Trusses,
        M1Baffle,
        MirrorCovers,
        LaserGuideStars,
        CRings,
        GIR,
        LPA,
        Platforms,*/
    ];

    let mut fem = fem::FEM::from_env()?;
    println!("{}", fem);
    fem.keep_inputs(&[0])
        .filter_inputs_by(&[0], |x| {
            loads
                .iter()
                .flat_map(|x| x.fem())
                .fold(false, |b, p| b || x.descriptions.contains(&p))
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

    let mut cfd_loads =
        windloads::CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency as usize)
            .duration(sim_duration)
            .nodes(loads.iter().flat_map(|x| x.keys()).collect(), locations)
            //.m1_segments()
            //.m2_segments()
            .build()
            .unwrap();

    let (mut cfd_source, mut sampler, mut sink) = stage!(Vec<f64>: source >> sampler << sink);

    channel!(cfd_source => sink; 1);

    let mut logging = Logging::default();
    println!("Starting the model");
    let now = Instant::now();
    spawn!((cfd_source, cfd_loads,)); //, (sampler, Sampler::default(),));
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

    let fxyz: Vec<_> = logging
        .deref()
        .iter()
        .map(|x| {
            let forces = x.chunks(3).step_by(2).flatten().collect::<Vec<&f64>>();
            forces.iter().step_by(3).fold(0f64, |a, &&x| a + x)
        })
        /*
                .chain(logging.deref().iter().map(|x| {
                    let forces = x.chunks(3).step_by(2).flatten().collect::<Vec<&f64>>();
                    forces.iter().skip(1).step_by(3).fold(0f64, |a, &&x| a + x)
                }))
                .chain(logging.deref().iter().map(|x| {
                    let forces = x.chunks(3).step_by(2).flatten().collect::<Vec<&f64>>();
                    forces.iter().skip(2).step_by(3).fold(0f64, |a, &&x| a + x)
                }))
        */
        .collect();
    dbg!(fxyz.len());
    let welch: PowerSpectrum<f64> = PowerSpectrum::builder(&fxyz)
        //.n_signal(3)
        .sampling_frequency(sim_sampling_frequency)
        .build();
    let psd = welch.periodogram();

    let x_psd: Vec<f64> = psd
        .chunks(psd.len() / 1)
        .take(1)
        .flat_map(|x| x.to_vec())
        .collect();
    /*
        let y_psd: Vec<f64> = psd
            .chunks(psd.len() / 3)
            .skip(1)
            .take(1)
            .flat_map(|x| x.to_vec())
            .collect();
        let z_psd: Vec<f64> = psd
            .chunks(psd.len() / 3)
            .skip(2)
            .take(1)
            .flat_map(|x| x.to_vec())
            .collect();
    */
    let _: complot::LogLog = (
        psd.frequency()
            .into_iter()
            .zip(x_psd.iter()) //.zip(&y_psd).zip(&z_psd))
            .skip(1)
            .map(|(f, x)| (f, vec![*x])),
        complot::complot!(
            "examples/figures/windloads_x-psd.png",
            xlabel = "Frequency [Hz]",
            ylabel = "PSD [N^2/Hz]"
        ),
    )
        .into();

    Ok(())
}
