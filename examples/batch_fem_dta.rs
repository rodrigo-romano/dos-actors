use dos_actors::clients::mount::{Mount, MountEncoders, MountTorques};
use dos_actors::{
    clients::{arrow_client::Arrow, dta},
    prelude::*,
};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use parse_monitors::cfd;
use std::{env, fs::create_dir, path::Path};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000_usize;
    let sim_duration = 400;
    const CFD_RATE: usize = 50;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE;

    let fem_env = env::var("FEM_REPO")?;
    let fem_name = Path::new(&fem_env)
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap();

    for cfd_case in cfd::Baseline::<2021>::mount().into_iter().skip(3).take(1) {
        println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
        let cfd_path = Path::new("/fsx/DTA").join(cfd_case.to_string());
        let data_path = cfd_path.join(fem_name);
        if !data_path.is_dir() {
            create_dir(&data_path)?
        }

        let mut fem = FEM::from_env()?; //.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());

        let state_space = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .max_eigen_frequency(75f64)
            //.truncate_hankel_singular_values(1e-5)
            .ins::<OSSElDriveTorque>()
            .ins::<OSSAzDriveTorque>()
            .ins::<OSSRotDriveTorque>()
            .ins::<OSSDTAWind6F>()
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .outs::<PMT3D>()
            //.use_static_gain_compensation(n_io)
            .build()?;

        let cfd_loads =
            dta::CfdLoads::new(cfd_path.join("GMT-DTA-190952_RevB1_WLC00xx.csv")).unwrap();

        let mut source: Initiator<_, CFD_RATE> = cfd_loads.into();

        let n_step = sim_duration * sim_sampling_frequency;
        let logging = Arrow::builder(n_step)
            .filename(
                data_path
                    .join("windloading.dta.parquet")
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
            .build()
            .into_arcx();
        let mut sink = Terminator::<_>::new(logging.clone());

        let mut sampler: Actor<_, CFD_RATE, 1> = Sampler::default().into();

        type D = Vec<f64>;
        let mut fem: Actor<_> = state_space.into();
        source
            .add_output()
            .build::<D, OSSDTAWind6F>()
            .into_input(&mut sampler);
        sampler
            .add_output()
            .build::<D, OSSDTAWind6F>()
            .into_input(&mut fem);

        let mut mount: Actor<_> = Mount::new().into();

        fem.add_output()
            .bootstrap()
            .build::<D, MountEncoders>()
            .into_input(&mut mount)
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
            Box::new(sampler),
            Box::new(mount),
            Box::new(fem),
            Box::new(sink),
        ])
        .name("batch-fem-dta")
        .check()?
        .flowchart()
        .run()
        .wait()
        .await?;
    }

    Ok(())
}
