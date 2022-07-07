use dos_actors::{
    clients::{
        arrow_client::Arrow,
        mount::{Mount, MountEncoders, MountSetPoint, MountTorques},
        windloads,
    },
    prelude::*,
};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use parse_monitors::cfd;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pwd = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("wind_loading");
    std::env::set_var("DATA_REPO", &pwd);

    let sim_sampling_frequency = 1000_usize;

    let sim_duration = 10f64;
    log::info!("Simulation duration: {:6.3}s", sim_duration);

    let (cfd_loads, state_space) = {
        use dos_actors::clients::windloads::WindLoads::*;
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
        let mut fem = FEM::from_env()?.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());
        println!("{}", fem);
        //println!("{}", fem);
        let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
        println!("CFD CASE (20Hz): {}", cfd_case);
        let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

        let cfd_loads =
            windloads::CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency)
                .duration(sim_duration as f64)
                //.time_range((200f64, 340f64))
                //.nodes(loads.iter().flat_map(|x| x.keys()).collect(), locations)
                .loads(loads, &mut fem, 0)
                .m1_segments()
                .m2_segments()
                .build()
                .unwrap()
                .into_arcx();

        (cfd_loads, {
            DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
                .sampling(sim_sampling_frequency as f64)
                .proportional_damping(2. / 100.)
                //.truncate_hankel_singular_values(1e-4)
                //.max_eigen_frequency(75.)
                //.use_static_gain_compensation(n_io)
                .ins::<CFD2021106F>()
                .ins::<OSSElDriveTorque>()
                .ins::<OSSAzDriveTorque>()
                .ins::<OSSRotDriveTorque>()
                .outs::<OSSAzEncoderAngle>()
                .outs::<OSSElEncoderAngle>()
                .outs::<OSSRotEncoderAngle>()
                .outs::<OSSM1Lcl>()
                .outs::<MCM2Lcl6D>()
                .build()?
                .into_arcx()
        })
    };
    println!("{}", *cfd_loads.lock().await);
    println!("{}", *state_space.lock().await);
    //println!("Y sizes: {:?}", state_space.y_sizes);

    let n_step = (sim_duration * sim_sampling_frequency as f64) as usize;
    let logging = Arrow::builder(n_step).build().into_arcx();
    let mnt_ctrl = Mount::at_zenith_angle(30)?.into_arcx();

    let mut source: Initiator<_> = Actor::new(cfd_loads.clone());
    let mut sink = Terminator::<_>::new(logging.clone());
    // FEM
    let mut fem: Actor<_> = Actor::new(state_space.clone());
    // MOUNT
    let mut mount: Actor<_> = Actor::new(mnt_ctrl.clone());

    source
        .add_output()
        .build::<CFD2021106F>()
        .into_input(&mut fem);
    source
        .add_output()
        .build::<OSSM1Lcl6F>()
        .into_input(&mut fem);
    source
        .add_output()
        .build::<MCM2LclForce6F>()
        .into_input(&mut fem);

    let mut mount_set_point: Initiator<_> = Signals::new(3, n_step).into();
    mount_set_point
        .add_output()
        .build::<MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_output()
        .build::<MountTorques>()
        .into_input(&mut fem);

    fem.add_output()
        .bootstrap()
        .multiplex(2)
        .build::<MountEncoders>()
        .into_input(&mut mount)
        .logn(&mut sink, 14)
        .await
        .confirm()?;
    fem.add_output()
        .bootstrap()
        .build::<OSSM1Lcl>()
        .logn(&mut sink, 42)
        .await;
    fem.add_output()
        .bootstrap()
        .build::<MCM2Lcl6D>()
        .logn(&mut sink, 42)
        .await;

    Model::new(vec![
        Box::new(source),
        Box::new(mount_set_point),
        Box::new(fem),
        Box::new(mount),
        Box::new(sink),
    ])
    .name("wind_loading")
    .flowchart()
    .check()?
    .run()
    .wait()
    .await?;

    Ok(())
}
