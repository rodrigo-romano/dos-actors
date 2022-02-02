use dos_actors::prelude::*;
use dosio::ios;
use fem::{
    dos::{DiscreteModalSolver, Exponential},
    FEM,
};
use m1_ctrl as m1;
use mount_ctrl as mount;
use std::{ops::Deref, time::Instant};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //simple_logger::SimpleLogger::new().env().init().unwrap();

    let sim_sampling_frequency = 1000;
    let sim_duration = 10_usize;

    // MOUNT
    let mut mnt_ctrl = mount::controller::Controller::new();
    let mut mnt_driver = mount::drives::Controller::new();
    // M1
    let mut hardpoints = m1::hp_dynamics::Controller::new();
    let mut load_cells = m1::hp_load_cells::Controller::new();
    let mut actuators = m1::actuators::segment1::Controller::new();
    // FEM
    let mut state_space = {
        let mut fem = FEM::from_env()?;
        println!("{}", fem);
        //let ins: Vec<_> = (1..=7).chain(once(14)).collect();
        //let outs: Vec<_> = (2..=8).chain(23..=24).collect();
        fem.keep_inputs(&[1, 11, 12, 14, 16])
            .keep_outputs(&[20, 21, 23, 24, 25]);
        println!("{}", fem);
        DiscreteModalSolver::<Exponential>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .inputs_from(&[&mnt_driver])
            .inputs_from(&[&hardpoints])
            .inputs(vec![ios!(M1ActuatorsSegment1)])
            .outputs(vec![ios!(OSSM1Lcl)])
            .outputs(ios!(
                OSSAzEncoderAngle,
                OSSElEncoderAngle,
                OSSRotEncoderAngle
            ))
            .outputs(vec![ios!(OSSHardpointD)])
            .build()?
    };
    println!("{}", state_space);

    println!("Y sizes: {:?}", state_space.y_sizes);

    const M1_RATE: usize = 10;

    let mut mount_controller = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("Mount Ctrlr");
    let mut mount_driver = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("Mount Driver");

    let mut rbm_cmd = Initiator::<Vec<f64>, 1>::build().tag("RBM Cmd");
    let mut bm_cmd = Initiator::<Vec<f64>, M1_RATE>::build().tag("BM Cmd");
    let mut m1_hardpoints = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("M1 hardpoints");
    let mut m1_hp_loadcells =
        Actor::<Vec<f64>, Vec<f64>, 1, M1_RATE>::new().tag("M1 hardpoints load cells");
    let mut m1s1 = Actor::<Vec<f64>, Vec<f64>, M1_RATE, 1>::new().tag("M1 S1");
    let mut fem = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("FEM");
    let mut sink = Terminator::<Vec<f64>, 1>::build().tag("sink");

    channel![mount_controller => mount_driver];
    channel![mount_driver => fem; 3];

    channel![fem => sink];
    channel![fem => (mount_controller,mount_driver); 3];

    channel![rbm_cmd => m1_hardpoints];
    channel![fem => m1_hp_loadcells => m1s1];
    channel![bm_cmd => m1s1];
    channel![m1_hardpoints => (fem, m1_hp_loadcells); 1];
    channel![m1s1 => fem];

    let n_iterations = sim_sampling_frequency * sim_duration;
    let mut signals =
        Signals::new(vec![42], n_iterations).output_signal(0, 0, Signal::Constant(1e-6));
    spawn!(
        (rbm_cmd, signals,),
        (bm_cmd, Signals::new(vec![27], n_iterations),),
        (m1_hardpoints, hardpoints,),
        (m1_hp_loadcells, load_cells,),
        (m1s1, actuators, vec![vec![0f64; 335]]),
        (
            fem,
            state_space,
            vec![
                vec![0f64; 42],
                vec![0f64; 4],
                vec![0f64; 6],
                vec![0f64; 4],
                vec![0f64; 84],
            ]
        ),
        (mount_controller, mnt_ctrl, vec![vec![0f64; 3]]),
        (mount_driver, mnt_driver,)
    );

    println!("Starting the model");
    let now = Instant::now();

    let mut logging = Logging::default();
    run!(sink, logging);

    let elapsed = now.elapsed().as_millis();
    println!("Model run {}s in {}ms ()", sim_duration, elapsed);

    let tau = (sim_sampling_frequency as f64).recip();
    let _: complot::Plot = (
        logging.deref().iter().enumerate().map(|(i, x)| {
            (
                i as f64 * tau,
                x.iter().map(|x| x * 1e6).collect::<Vec<f64>>(),
            )
        }),
        complot::complot!(
            "examples/figures/m1_rbm_ctrl.png",
            xlabel = "Time [s]",
            ylabel = ""
        ),
    )
        .into();

    Ok(())
}
