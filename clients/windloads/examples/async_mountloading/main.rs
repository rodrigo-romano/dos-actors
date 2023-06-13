use std::{env, fs::DirBuilder, path::Path};

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{OneSignal, Signal, Signals, Smooth, Weight};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{
    fem_io::{
        actors_inputs::{MCM2Lcl6F, OSSM1Lcl6F, CFD2021106F},
        actors_outputs::{MCM2Lcl6D, OSSM1Lcl},
    },
    DiscreteModalSolver, ExponentialMatrix,
};
use gmt_dos_clients_io::{
    cfd_wind_loads::{CFDM1WindLoads, CFDM2WindLoads, CFDMountWindLoads},
    gmt_m1::M1RigidBodyMotions,
    gmt_m2::M2RigidBodyMotions,
};
use gmt_dos_clients_mount::Mount;
use gmt_dos_clients_windloads::CfdLoads;
use gmt_fem::FEM;
use parse_monitors::cfd;

async fn task(
    cfd_path: &Path,
    sim_sampling_frequency: usize,
    sim_duration: usize,
    data_repo: &Path,
) -> anyhow::Result<()> {
    let n_step = sim_sampling_frequency * sim_duration;

    // GMT FEM
    let mut fem = FEM::from_env()?;

    // CFD WIND LOADS

    let cfd_loads_client = CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency)
        .duration(sim_duration as f64)
        .mount(&mut fem, 0, None)
        .m1_segments()
        .m2_segments()
        .build()?
        .into_arcx();

    // FEM STATE SPACE
    let state_space = {
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            //.max_eigen_frequency(75f64)
            .including_mount()
            .ins::<CFD2021106F>()
            .ins::<OSSM1Lcl6F>()
            .ins::<MCM2Lcl6F>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .use_static_gain_compensation()
            .build()?
    };

    // SET POINT
    let mut setpoint: Initiator<_> = Signals::new(3, n_step).into();
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT CONTROL
    // let mut mount: Actor<_> = Mount::new().into();
    let mount: Actor<_> = Mount::builder(&mut setpoint).build(&mut fem)?;
    // Logger
    let logging = Arrow::builder(n_step)
        .filename(data_repo.join("windloading").to_str().unwrap())
        .build()
        .into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let mut cfd_loads: Initiator<_> = Actor::new(cfd_loads_client.clone()).name("CFD Wind loads");
    let signals = Signals::new(1, n_step).channel(
        0,
        Signal::Sigmoid {
            amplitude: 1f64,
            sampling_frequency_hz: sim_sampling_frequency as f64,
        },
    );
    let signal = OneSignal::try_from(signals)?.into_arcx();
    let m1_smoother = Smooth::new().into_arcx();
    let m2_smoother = Smooth::new().into_arcx();
    let mount_smoother = Smooth::new().into_arcx();

    let mut sigmoid: Initiator<_> = Actor::new(signal.clone()).name("Sigmoid");
    let mut smooth_m1_loads: Actor<_> = Actor::new(m1_smoother.clone());
    let mut smooth_m2_loads: Actor<_> = Actor::new(m2_smoother.clone());
    let mut smooth_mount_loads: Actor<_> = Actor::new(mount_smoother.clone());

    sigmoid
        .add_output()
        .multiplex(3)
        .build::<Weight>()
        .into_input(&mut smooth_m1_loads)
        .into_input(&mut smooth_m2_loads)
        .into_input(&mut smooth_mount_loads)?;
    cfd_loads
        .add_output()
        .build::<CFDM1WindLoads>()
        .into_input(&mut smooth_m1_loads)?;
    smooth_m1_loads
        .add_output()
        .build::<CFDM1WindLoads>()
        .into_input(&mut fem)?;
    cfd_loads
        .add_output()
        .build::<CFDM2WindLoads>()
        .into_input(&mut smooth_m2_loads)?;
    smooth_m2_loads
        .add_output()
        .build::<CFDM2WindLoads>()
        .into_input(&mut fem)?;
    cfd_loads
        .add_output()
        .build::<CFDMountWindLoads>()
        .into_input(&mut smooth_mount_loads)?;
    smooth_mount_loads
        .add_output()
        .build::<CFDMountWindLoads>()
        .into_input(&mut fem)?;

    /*     fem.add_output()
           .bootstrap()
           .build::<MountEncoders>()
           .logn(&mut sink, 14)
           .await?;
    */
    fem.add_output()
        .unbounded()
        .build::<M1RigidBodyMotions>()
        .log(&mut sink)
        .await?;
    fem.add_output()
        .unbounded()
        .build::<M2RigidBodyMotions>()
        .log(&mut sink)
        .await?;

    model!(
        setpoint,
        mount,
        cfd_loads,
        sigmoid,
        smooth_m1_loads,
        smooth_m2_loads,
        smooth_mount_loads,
        fem,
        sink
    )
    .name("mountloading")
    .quiet()
    .check()?
    // .flowchart()
    .run()
    .wait()
    .await?;

    Ok(())
}

use indicatif::ProgressIterator;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000;
    let sim_duration = 400_usize; // second

    let fem_repo = env::var("FEM_REPO").expect("FEM_REPO env var missing");
    let fem_model = Path::new(&fem_repo).components().last().unwrap();
    let cfd_repo = env::var("CFD_REPO").expect("CFD_REPO env var missing");

    let mut handles = vec![];

    for cfd_case in cfd::Baseline::<2021>::default().into_iter() {
        let cfd_path = Path::new(&cfd_repo).join(cfd_case.to_string());
        let data_repo = cfd_path.join(&fem_model);
        if !data_repo.is_dir() {
            DirBuilder::new().recursive(true).create(&data_repo)?;
        }
        let h: tokio::task::JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
            task(&cfd_path, sim_sampling_frequency, sim_duration, &data_repo).await?;
            Ok(())
        });
        handles.push(h);
    }

    for handle in handles.into_iter().progress() {
        let _ = handle.await?;
    }

    Ok(())
}
