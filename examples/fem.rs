use dos_actors::{
    clients::{arrow_client::Arrow, windloads},
    prelude::*,
};
use dosio::ios;
use fem::{
    dos::{DiscreteModalSolver, Exponential},
    FEM,
};
use mount_ctrl as mount;
use parse_monitors::cfd;
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //simple_logger::SimpleLogger::new().env().init().unwrap();

    let sim_sampling_frequency = 1000;
    let sim_duration = 400_usize;
    const CFD_RATE: usize = 50;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE;

    let mut mnt_ctrl = mount::controller::Controller::new();
    let mut mnt_driver = mount::drives::Controller::new();

    let mut state_space = {
        let mut fem = FEM::from_env()?;
        println!("{}", fem);
        fem.keep_inputs(&[0, 11, 12, 16])
            .filter_inputs_by(&[0], |x| x.descriptions.contains("mirror cover"))
            .keep_outputs(&[19, 20, 21, 24, 25]);
        println!("{}", fem);
        DiscreteModalSolver::<Exponential>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .inputs(vec![ios!(CFD2021106F)])
            .inputs_from(&[&mnt_driver])
            .outputs(ios!(OSSM1Lcl, MCM2Lcl6D))
            .outputs(ios!(
                OSSAzEncoderAngle,
                OSSElEncoderAngle,
                OSSRotEncoderAngle
            ))
            .build()?
    };
    println!("{}", state_space);

    println!("Y sizes: {:?}", state_space.y_sizes);

    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
    let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

    let mut cfd_loads = windloads::CfdLoads::builder(cfd_path.to_str().unwrap())
        .duration(sim_duration)
        .keys(windloads::WindLoads::MirrorCovers.keys())
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
        stage!(Vec<f64>: (CFD[CFD_RATE] => sampler) FEM, Mount_Ctrlr, Mount_Driver << Logs);

    channel![source => sampler => fem];
    channel![fem => sink; 2];
    channel![mount_controller => mount_driver];
    channel![mount_driver => fem; 3];
    channel![fem => (mount_controller,mount_driver); 3];

    println!("{source}{fem}{mount_controller}{mount_driver}{sink}");

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

    let n_step = 1 + sim_duration * sim_sampling_frequency;
    let mut logging = Arrow::new(n_step, vec!["m1 rbm", "m2 rbm"], vec![42, 42]);
    run!(sink, logging);
    println!("Model run in {}ms", now.elapsed().as_millis());

    println!("{logging}");
    logging.to_parquet("data.parquet")?;

    Ok(())
}
