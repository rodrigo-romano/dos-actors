use std::{env, path::Path};
use nalgebra::{DMatrix, DVector};

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{OneSignal, Signal, Signals, Smooth, Weight};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{
    fem_io::{
        actors_inputs::{MCM2Lcl6F, OSSM1Lcl6F, CFD2021106F, OSSGIRTooth6F},
        actors_outputs::{MCM2Lcl6D, OSSM1Lcl, OSSGIR6d, OSSPayloads6D},
    },
    DiscreteModalSolver, ExponentialMatrix,
};
use gmt_dos_clients_io::{
    cfd_wind_loads::{CFDM1WindLoads, CFDM2WindLoads, CFDMountWindLoads},
    gmt_m1::M1RigidBodyMotions,
    gmt_m2::M2RigidBodyMotions,
    mount::MountEncoders,
};
use gmt_dos_clients_mount::Mount;
use gmt_dos_clients_windloads::CfdLoads;
use gmt_fem::FEM;
use parse_monitors::cfd;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 1000;
    let sim_duration = 2_usize; //1_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    // GMT FEM
    let mut fem = FEM::from_env()?;
    println!("{fem}");
    fem.filter_outputs_by(&[26], |x| 
        x.descriptions.contains("Instrument at Direct Gregorian Port B (employed)"));
    println!("{fem}");
        
    // CFD WIND LOADS
    let cfd_repo = env::var("CFD_REPO").expect("CFD_REPO env var missing");
    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    let path = Path::new(&cfd_repo).join(cfd_case.to_string());
    let cfd_loads_client = CfdLoads::foh(path.to_str().unwrap(), sim_sampling_frequency)
        .duration(sim_duration as f64)
        .mount(&mut fem, 0, None)
        .m1_segments()
        .m2_segments()
        .build()?
        .into_arcx();

    // FEM STATE SPACE
    let gir_tooth_axfo = DVector::kronecker(
        &DVector::from_vec(vec![1., -1., 1., -1., 1., -1., 1., -1.]),
        &DVector::from_vec(vec![0., 0., 0.25, 0., 0., 0.]));
    let state_space = {
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            //.max_eigen_frequency(75f64)
            .including_mount()
            .ins::<CFD2021106F>()
            .ins_with::<OSSGIRTooth6F>(gir_tooth_axfo.as_view())
            .ins::<OSSM1Lcl6F>()
            .ins::<MCM2Lcl6F>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .outs::<OSSGIR6d>()
            .outs::<OSSPayloads6D>()
            .use_static_gain_compensation()
            .build()?
    };
    println!("{state_space}");

    // SET POINT
    let mut setpoint: Initiator<_> = Signals::new(3, n_step).into();
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT CONTROL
    let mut mount: Actor<_> = Mount::builder(&mut setpoint).build(&mut fem)?;
    // Logger
    let logging = Arrow::builder(n_step).filename("examples/mountloading/mnt-wl_data.parquet").build().into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let mut cfd_loads: Initiator<_> = Actor::new(cfd_loads_client.clone()).name("CFD Wind loads");
    let mut signals = Signals::new(1, n_step).channel(
        0,
        Signal::Sigmoid {
            amplitude: 1f64,
            sampling_frequency_hz: sim_sampling_frequency as f64,
        },
    );
    signals.progress();
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

    mount.add_output()
        .build::<OSSGIRTooth6F>()
        .into_input(&mut fem)?;

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
    fem.add_output()
        .unbounded()
        .build::<MountEncoders>()
        .logn(&mut sink, 14)
        .await?;    
    fem.add_output()
        .unbounded()
        .build::<OSSGIR6d>()
        .logn(&mut sink, 6)
        .await?;
    fem.add_output()
        .unbounded()
        .build::<OSSPayloads6D>()
        .logn(&mut sink, 6)
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
    .check()?
    .flowchart()
    .run()
    .wait()
    .await?;

    Ok(())
}
