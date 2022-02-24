use dos_actors::{
    clients::{
        arrow_client::Arrow,
        windloads,
        windloads::{WindLoads::*, CS},
    },
    prelude::*,
};
use dosio::ios;
use fem::{
    dos::{DiscreteModalSolver, Exponential},
    FEM,
};
use mount_ctrl as mount;
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

    let mut mnt_ctrl = mount::controller::Controller::new();
    let mut mnt_driver = mount::drives::Controller::new();

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
    fem.keep_inputs(&[0, 10, 11, 12, 15, 16])
        .filter_inputs_by(&[0], |x| {
            loads
                .iter()
                .flat_map(|x| x.fem())
                .fold(false, |b, p| b || x.descriptions.contains(&p))
        })
        .keep_outputs(&[19, 20, 21, 24, 25]);
    println!("{}", fem);
    let locations: Vec<CS> = fem.inputs[0]
        .as_ref()
        .unwrap()
        .get_by(|x| Some(CS::OSS(x.properties.location.as_ref().unwrap().clone())))
        .into_iter()
        .step_by(6)
        .collect();

    let mut state_space = DiscreteModalSolver::<Exponential>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        .inputs(ios!(CFD2021106F, OSSM1Lcl6F, MCM2LclForce6F))
        //.inputs(vec![ios!(OSSM1Lcl6F)])
        .inputs_from(&[&mnt_driver])
        .outputs(ios!(OSSM1Lcl, MCM2Lcl6D))
        .outputs(ios!(
            OSSAzEncoderAngle,
            OSSElEncoderAngle,
            OSSRotEncoderAngle
        ))
        .build()?;
    println!("{}", state_space);

    println!("Y sizes: {:?}", state_space.y_sizes);

    let cfd_case =
        cfd::CfdCase::<2021>::colloquial(opt.zenith, opt.azimuth, &opt.enclosure, opt.wind_speed)?;
    println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
    let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

    let mut cfd_loads =
        windloads::CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency)
            .duration(sim_duration)
            //.time_range((200f64, 340f64))
            .nodes(loads.iter().flat_map(|x| x.keys()).collect(), locations)
            .m1_segments()
            .m2_segments()
            .build()
            .unwrap();

    /*
    let mut source = Initiator::<Vec<f64>, CFD_RATE>::build().tag("source");
    let mut sampler = Actor::<Vec<f64>, Vec<f64>, CFD_RATE, 1>::new();
    let mut fem = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("FEM");
    let mut mount_controller = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("Mount Ctrlr");
    let mut mount_driver = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("Mount Driver");
    let mut sink = Terminator::<Vec<f64>, 1>::build().tag("sink");
    */

    let (mut source, mut sampler, mut fem, mut mount_controller, mut mount_driver, mut sink) =
        stage!(Vec<f64>: (CFD[CFD_RATE] => sampler), FEM, Mount_Ctrlr, Mount_Driver << Logs);

    channel![source => fem; 3];
    channel![fem => sink; 2];
    channel![mount_controller => mount_driver];
    channel![mount_driver => fem; 3];
    channel![fem => (mount_controller,mount_driver); 3];

    println!("{source}{sampler}{fem}{mount_controller}{mount_driver}{sink}");

    println!("Starting the model");
    let now = Instant::now();
    spawn!(
        (source, cfd_loads,),
        (sampler, Sampler::default(),),
        (
            fem,
            state_space,
            vec![
                vec![0f64; 42],
                vec![0f64; 42],
                vec![0f64; 4],
                vec![0f64; 6],
                vec![0f64; 4]
            ]
        ),
        (mount_controller, mnt_ctrl, vec![vec![0f64; 3]]),
        (mount_driver, mnt_driver,)
    );

    let n_step = 1 + (sim_duration * sim_sampling_frequency as f64) as usize;
    let mut logging = Arrow::new(n_step, vec!["m1 rbm", "m2 rbm"], vec![42, 42]);
    run!(sink, logging);

    //dbg!(&logging);
    println!("Model run in {}ms", now.elapsed().as_millis());

    /*
    let tau = (sim_sampling_frequency as f64).recip();
    let _: complot::Plot = (
        logging
            .deref()
            .iter()
            .enumerate()
            .map(|(i, x)| (i as f64 * tau, x.to_owned())),
        complot::complot!("examples/mount.png", xlabel = "Time [s]", ylabel = ""),
    )
        .into();
     */

    println!("{logging}");
    let fem_env = env::var("FEM_REPO")?;
    let fem_name = Path::new(&fem_env)
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap();
    let data_path = cfd_path.join(fem_name);
    if !data_path.is_dir() {
        create_dir(&data_path)?
    }
    logging.to_parquet(data_path.join("windloading.parquet"))?;

    Ok(())
}
