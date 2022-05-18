use dos_actors::clients::mount::{Mount, MountEncoders, MountTorques};
use dos_actors::{
    clients::{
        arrow_client::Arrow,
        windloads,
        windloads::{WindLoads::*, CS},
    },
    prelude::*,
};
use fem::{
    dos::{DiscreteModalSolver, Exponential, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use parse_monitors::cfd;
use std::{env, fs::create_dir, path::Path};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000_usize;
    let sim_duration = 400;
    const CFD_RATE: usize = 1;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE;

    let fem_env = env::var("FEM_REPO")?;
    let fem_name = Path::new(&fem_env)
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap();

    for cfd_case in cfd::Baseline::<2021>::mount().into_iter().skip(7).take(1) {
        println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
        let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
        let data_path = cfd_path.join(fem_name);
        if !data_path.is_dir() {
            create_dir(&data_path)?
        }

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

        let mut fem = FEM::from_env()?; //.static_from_env();
                                        //let n_io = (fem.n_inputs(), fem.n_outputs());
        fem.filter_inputs_by(&[0], |x| {
            loads
                .iter()
                .flat_map(|x| x.fem())
                .fold(false, |b, p| b || x.descriptions.contains(&p))
        });
        let locations: Vec<CS> = fem.inputs[0]
            .as_ref()
            .unwrap()
            .get_by(|x| Some(CS::OSS(x.properties.location.as_ref().unwrap().clone())))
            .into_iter()
            .step_by(6)
            .collect();

        let state_space = DiscreteModalSolver::<Exponential>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .max_eigen_frequency(75f64)
            //.truncate_hankel_singular_values(1e-5)
            .ins::<OSSElDriveTorque>()
            .ins::<OSSAzDriveTorque>()
            .ins::<OSSRotDriveTorque>()
            .ins::<CFD2021106F>()
            .ins::<OSSM1Lcl6F>()
            .ins::<MCM2LclForce6F>()
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .outs::<PMT3D>()
            //.use_static_gain_compensation(n_io)
            .build()?;

        let cfd_loads =
            windloads::CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency)
                .duration(sim_duration as f64)
                .nodes(loads.iter().flat_map(|x| x.keys()).collect(), locations)
                .m1_segments()
                .m2_segments()
                .build()
                .unwrap();

        let mut source: Initiator<_> = cfd_loads.into();

        let n_step = sim_duration * sim_sampling_frequency;
        let logging = Arrow::builder(n_step)
            .filename(
                data_path
                    .join("windloading-30s.parquet")
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
            .build()
            .into_arcx();
        let mut sink = Terminator::<_>::new(logging.clone());

        type D = Vec<f64>;
        let mut fem: Actor<_> = state_space.into();
        source
            .add_output()
            .build::<D, CFD2021106F>()
            .into_input(&mut fem);
        source
            .add_output()
            .build::<D, OSSM1Lcl6F>()
            .into_input(&mut fem);
        source
            .add_output()
            .build::<D, MCM2LclForce6F>()
            .into_input(&mut fem);

        let mut mount: Actor<_> = Mount::new().into();

        fem.add_output()
            .bootstrap()
            .multiplex(2)
            .build::<D, MountEncoders>()
            .into_input(&mut mount)
            .log(&mut sink, 14)
            .await
            .confirm()?
            .add_output()
            .build::<D, OSSM1Lcl>()
            .log(&mut sink, 42)
            .await
            .confirm()?
            .add_output()
            .build::<D, MCM2Lcl6D>()
            .log(&mut sink, 42)
            .await
            .confirm()?
            .add_output()
            .build::<D, PMT3D>()
            .log(&mut sink, 300)
            .await
            .confirm()?;

        mount
            .add_output()
            .multiplex(2)
            .build::<D, MountTorques>()
            .into_input(&mut fem)
            .log(&mut sink, 20)
            .await
            .confirm()?;

        Model::new(vec![
            Box::new(source),
            Box::new(mount),
            Box::new(fem),
            Box::new(sink),
        ])
        .name("batch-fem")
        .check()?
        .flowchart()
        .run()
        .wait()
        .await?;
    }

    Ok(())
}
