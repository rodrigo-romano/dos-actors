use dos_actors::{
    clients::{
        windloads::{CfdLoads, M1Loads, M2Loads, MountLoads, WindLoads::*, CS},
        Logging, Sampler,
    },
    prelude::*,
};
use parse_monitors::cfd;
use std::time::Instant;
use welch_sde::{Build, PowerSpectrum, Welch};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000f64;
    let sim_duration = 30f64;
    const CFD_RATE: usize = 1;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE as f64;
    /*assert_eq!(
        cfd_sampling_frequency, 20f64,
        "Expected 20Hz, found {}",
        cfd_sampling_frequency
    );*/

    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
    let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

    let loads = vec![
        TopEnd,
        M2Baffle,
        Trusses,
        M1Baffle,
        MirrorCovers,
        LaserGuideStars,
        CRings,
        GIR,
        Platforms,
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

    let cfd_loads = CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency as usize)
        //let mut cfd_loads = CfdLoads::zoh(cfd_path.to_str().unwrap())
        .duration(sim_duration)
        .nodes(loads.iter().flat_map(|x| x.keys()).collect(), locations)
        .m1_segments()
        .m2_segments()
        .build()
        .unwrap();

    let mut source: Initiator<_> = cfd_loads.into();

    let logging = Logging::<f64>::default().n_entry(3).into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let buffer_cap = Some(vec![
        sim_sampling_frequency as usize * sim_duration as usize,
    ]);

    let mut mount_loads: Actor<_> = Sampler::<Vec<f64>, MountLoads>::default().into();
    source
        .add_output::<Vec<f64>, MountLoads>(buffer_cap.clone())
        .into_input(&mut mount_loads);
    mount_loads
        .add_output::<Vec<f64>, MountLoads>(buffer_cap.clone())
        .into_input(&mut sink);

    let mut m1_loads: Actor<_> = Sampler::<Vec<f64>, M1Loads>::default().into();
    source
        .add_output::<Vec<f64>, M1Loads>(buffer_cap.clone())
        .into_input(&mut m1_loads);
    m1_loads
        .add_output::<Vec<f64>, M1Loads>(buffer_cap.clone())
        .into_input(&mut sink);

    let mut m2_loads: Actor<_> = Sampler::<Vec<f64>, M2Loads>::default().into();
    source
        .add_output::<Vec<f64>, M2Loads>(buffer_cap.clone())
        .into_input(&mut m2_loads);
    m2_loads
        .add_output::<Vec<f64>, M2Loads>(buffer_cap.clone())
        .into_input(&mut sink);

    println!("Starting the model");
    let now = Instant::now();
    spawn!(source, mount_loads, m1_loads, m2_loads);
    run!(sink);
    println!("{}", *logging.lock().await);
    println!("Model run in {}ms", now.elapsed().as_millis());

    let logs = &*logging.lock().await;
    let forces = logs.chunks().map(|x| {
        let forces = x.chunks(3).step_by(2).flatten().collect::<Vec<&f64>>();
        let fx = forces.iter().step_by(3).fold(0f64, |a, &&x| a + x);
        let fy = forces.iter().skip(1).step_by(3).fold(0f64, |a, &&x| a + x);
        let fz = forces.iter().skip(2).step_by(3).fold(0f64, |a, &&x| a + x);
        vec![fx, fy, fz]
    });

    let _: complot::Plot = (
        forces
            .enumerate()
            .map(|(i, x)| (i as f64 * sim_sampling_frequency.recip(), x)),
        complot::complot!(
            "examples/figures/windloads-obj.png",
            xlabel = "Time [s]",
            ylabel = "Force Mag. [N]"
        ),
    )
        .into();

    /*
        let fxyz: Vec<_> = (*logging.lock().await)
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
                "examples/figures/windloads_x-psd-obj.png",
                xlabel = "Frequency [Hz]",
                ylabel = "PSD [N^2/Hz]"
            ),
        )
            .into();
    */
    Ok(())
}
