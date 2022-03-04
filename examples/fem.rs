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
    dos::{DiscreteModalSolver, Exponential},
    fem_io::*,
    FEM,
};
use parse_monitors::cfd;
use std::{env, fs::create_dir, path::Path, time::Instant};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "GMT Linear Optical Model",
    about = "GMT M1/M2 rigid body motions to optics linear transformations"
)]
struct Opt {
    /// CFD zenith angle
    #[structopt(short, long)]
    zenith: u32,
    /// CFD azimuth angle
    #[structopt(short, long)]
    azimuth: u32,
    /// CFD enclosure configuration
    #[structopt(short, long)]
    enclosure: String,
    /// CFD wind speed
    #[structopt(short, long)]
    wind_speed: u32,
    /// Simulation duration [s]
    #[structopt(short, long, default_value = "400")]
    duration: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    //simple_logger::SimpleLogger::new().env().init().unwrap();

    let sim_sampling_frequency = 1000_usize;
    let sim_duration = opt.duration;
    const CFD_RATE: usize = 1;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE;

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

    let mut fem = FEM::from_env()?;
    println!("{}", fem);
    /*fem.keep_inputs(&[0, 10, 11, 12, 15, 16])
    .filter_inputs_by(&[0], |x| {
        loads
            .iter()
            .flat_map(|x| x.fem())
            .fold(false, |b, p| b || x.descriptions.contains(&p))
    })
    .keep_outputs(&[19, 20, 21, 24, 25]);*/
    fem.filter_inputs_by(&[0], |x| {
        loads
            .iter()
            .flat_map(|x| x.fem())
            .fold(false, |b, p| b || x.descriptions.contains(&p))
    });
    println!("{}", fem);
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
        .build()?;
    println!("{}", state_space);

    println!("Y sizes: {:?}", state_space.y_sizes);

    let cfd_case =
        cfd::CfdCase::<2021>::colloquial(opt.zenith, opt.azimuth, &opt.enclosure, opt.wind_speed)?;
    println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
    let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

    let cfd_loads = windloads::CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency)
        .duration(sim_duration)
        //.time_range((200f64, 340f64))
        .nodes(loads.iter().flat_map(|x| x.keys()).collect(), locations)
        .m1_segments()
        .m2_segments()
        .build()
        .unwrap();

    let mut source: Initiator<_> = cfd_loads.into();

    let n_step = (sim_duration * sim_sampling_frequency as f64) as usize;
    let logging = Arrow::<f64>::new(n_step)
        .entry::<OSSM1Lcl>(42)
        .entry::<MCM2Lcl6D>(42)
        .into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    type D = Vec<f64>;
    let mut fem: Actor<_> = state_space.into();
    source
        .add_output::<D, CFD2021106F>(None)
        .into_input(&mut fem);
    source
        .add_output::<D, OSSM1Lcl6F>(None)
        .into_input(&mut fem);
    source
        .add_output::<D, MCM2LclForce6F>(None)
        .into_input(&mut fem);

    let mut mount: Actor<_> = Mount::new().into();
    mount
        .add_output::<D, MountTorques>(None)
        .into_input(&mut fem);

    fem.add_output::<D, MountEncoders>(None)
        .into_input(&mut mount);
    fem.add_output::<D, OSSM1Lcl>(None).into_input(&mut sink);
    fem.add_output::<D, MCM2Lcl6D>(None).into_input(&mut sink);

    println!("Starting the model");
    let now = Instant::now();
    spawn!(source, mount);
    spawn_bootstrap!(fem::<D, MountEncoders>);
    run!(sink);
    println!("Model run in {}ms", now.elapsed().as_millis());

    println!("{}", *logging.lock().await);
    let fem_env = env::var("FEM_REPO")?;
    let fem_name = Path::new(&fem_env)
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap();
    let data_path = cfd_path.join(fem_name);
    if !data_path.is_dir() {
        create_dir(&data_path)?
    }
    //(*logging.lock().await).to_parquet(data_path.join("windloading.parquet"))?;
    (*logging.lock().await).to_parquet("data.parquet")?;

    Ok(())
}
