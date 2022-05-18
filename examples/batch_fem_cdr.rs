use dos_actors::clients::mount::{Mount, MountEncoders, MountTorques};
use dos_actors::{clients::arrow_client::Arrow, prelude::*};
use fem::{
    dos::{DiscreteModalSolver, Exponential, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use matio_rs::{Load, MatFile};
use parse_monitors::cfd;
use std::{env, fs::create_dir, path::Path};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000_usize;
    let sim_duration = 20;
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

        let mut fem = FEM::from_env()?; //.static_from_env();
                                        //let n_io = (fem.n_inputs(), fem.n_outputs());
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

        let mat_file = MatFile::load(&cfd_path.join("convertedToFEM.mat"))?;
        let mat = mat_file.read("inputFEM")?;
        let loads: Vec<f64> = mat.into();

        let n_step = sim_duration * sim_sampling_frequency;
        let n = 354;
        let (mount, (m1, m2)): (Vec<_>, (Vec<_>, Vec<_>)) = loads
            .chunks(n)
            .take(n_step)
            .map(|data| {
                let (mount, other) = data.split_at(270);
                let (m1, m2) = other.split_at(42);
                (mount.to_vec(), (m1.to_vec(), m2.to_vec()))
            })
            .unzip();
        const CFD_RATE: usize = 50;
        let mut mount_loads: Initiator<_, CFD_RATE> = (
            Source::new(mount.into_iter().flatten().collect::<Vec<f64>>(), 270),
            "Mount Loads",
        )
            .into();
        let mut m1_loads: Initiator<_, CFD_RATE> = (
            Source::new(m1.into_iter().flatten().collect::<Vec<f64>>(), 42),
            "M1 Loads",
        )
            .into();
        let mut m2_loads: Initiator<_, CFD_RATE> = (
            Source::new(m2.into_iter().flatten().collect::<Vec<f64>>(), 42),
            "M2 Loads",
        )
            .into();

        let logging = Arrow::builder(n_step)
            .filename(
                data_path
                    .join("windloading-20s.parquet")
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
            .build()
            .into_arcx();
        let mut sink = Terminator::<_>::new(logging.clone());
        let mut mount_loads_upsampler: Actor<_, CFD_RATE, 1> = Sampler::default().into();
        let mut m1_loads_upsampler: Actor<_, CFD_RATE, 1> = Sampler::default().into();
        let mut m2_loads_upsampler: Actor<_, CFD_RATE, 1> = Sampler::default().into();

        mount_loads
            .add_output()
            .build::<D, CFD2021106F>()
            .into_input(&mut &mut mount_loads_upsampler);
        m1_loads
            .add_output()
            .build::<D, OSSM1Lcl6F>()
            .into_input(&mut &mut m1_loads_upsampler);
        m2_loads
            .add_output()
            .build::<D, MCM2LclForce6F>()
            .into_input(&mut &mut m2_loads_upsampler);

        type D = Vec<f64>;
        let mut fem: Actor<_> = state_space.into();
        mount_loads_upsampler
            .add_output()
            .build::<D, CFD2021106F>()
            .into_input(&mut fem);
        m1_loads_upsampler
            .add_output()
            .build::<D, OSSM1Lcl6F>()
            .into_input(&mut fem);
        m2_loads_upsampler
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
            Box::new(mount_loads),
            Box::new(m1_loads),
            Box::new(m2_loads),
            Box::new(mount_loads_upsampler),
            Box::new(m1_loads_upsampler),
            Box::new(m2_loads_upsampler),
            Box::new(mount),
            Box::new(fem),
            Box::new(sink),
        ])
        .name("batch-fem-cdr")
        .check()?
        .flowchart()
        .run()
        .wait()
        .await?;
    }

    Ok(())
}
