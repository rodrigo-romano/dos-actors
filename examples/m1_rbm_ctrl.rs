use dos_actors::{clients, prelude::*};
use dosio::ios;
use fem::{
    dos::{DiscreteModalSolver, Exponential},
    FEM,
};
use m1_ctrl as m1;
use mount_ctrl as mount;
use std::{
    ops::Deref,
    thread,
    time::{Duration, Instant},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //simple_logger::SimpleLogger::new().env().init().unwrap();

    let sim_sampling_frequency = 1000;
    let sim_duration = 4_usize;

    // MOUNT
    let mut mnt_ctrl = mount::controller::Controller::new();
    let mut mnt_driver = mount::drives::Controller::new();
    // M1
    let mut hardpoints = m1::hp_dynamics::Controller::new();
    let mut load_cells = m1::hp_load_cells::Controller::new();
    //let mut m1s1_actuators = m1::actuators::segment1::Controller::new();
    //let mut m2s2_actuators = m1::actuators::segment2::Controller::new();
    // FEM
    let mut state_space = {
        let mut fem = FEM::from_env()?;
        println!("{}", fem);
        //let ins: Vec<_> = (1..=7).chain(once(14)).collect();
        //let outs: Vec<_> = (2..=8).chain(23..=24).collect();
        fem.keep_inputs(&[1, 2, 3, 4, 5, 6, 7, 11, 12, 14, 16])
            .keep_outputs(&[20, 21, 23, 24, 25]);
        println!("{}", fem);
        DiscreteModalSolver::<Exponential>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .inputs_from(&[&mnt_driver])
            .inputs_from(&[&hardpoints])
            .inputs(vec![ios!(M1ActuatorsSegment1)])
            .inputs(vec![ios!(M1ActuatorsSegment2)])
            .inputs(vec![ios!(M1ActuatorsSegment3)])
            .inputs(vec![ios!(M1ActuatorsSegment4)])
            .inputs(vec![ios!(M1ActuatorsSegment5)])
            .inputs(vec![ios!(M1ActuatorsSegment6)])
            .inputs(vec![ios!(M1ActuatorsSegment7)])
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

    let (mut rbm_cmd, mut mount_controller, mut mount_driver, mut m1_hardpoints, mut fem, mut sink) =
        stage!(Vec<f64>: RBM_Cmd >> Mount_Ctrlr, Mount_Driver, M1_Hardpoints, FEM << Sink);

    let mut m1_hp_loadcells =
        Actor::<Vec<f64>, Vec<f64>, 1, M1_RATE>::new().tag("M1 hardpoints load cells");

    let n_segment = 7;
    let mut sx_bm_cmd: Vec<_> = (1..=n_segment)
        .map(|sid| Initiator::<Vec<f64>, M1_RATE>::build().tag(format!("BM S{sid}")))
        .collect();

    channel![mount_controller => mount_driver];
    channel![mount_driver => fem; 3];

    channel![fem => sink];
    channel![fem => (mount_controller,mount_driver); 3];

    channel![rbm_cmd => m1_hardpoints];
    channel![fem => m1_hp_loadcells];
    channel![m1_hardpoints => (fem, m1_hp_loadcells)];

    let m1_assembly = clients::m1::assembly::Controller::new(
        &mut m1_hp_loadcells,
        sx_bm_cmd.as_mut_slice(),
        &mut fem,
    );

    let n_iterations = sim_sampling_frequency * sim_duration;
    let mut signals = (0..n_segment).fold(Signals::new(vec![42], n_iterations), |s, i| {
        (0..1).fold(s, |ss, j| {
            ss.output_signal(
                0,
                i * 6 + j,
                Signal::Constant((-1f64).powi(j as i32) * (1 + i) as f64 * 1e-6),
            )
        })
    });
    spawn!(
        (rbm_cmd, signals,),
        (m1_hardpoints, hardpoints,),
        (m1_hp_loadcells, load_cells,),
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
    for mut si_bm_cmd in sx_bm_cmd.into_iter() {
        spawn!((si_bm_cmd, Signals::new(vec![27], n_iterations),));
    }
    m1_assembly.spawn();

    println!("Starting the model");
    let now = Instant::now();

    let mut logging = Logging::default();
    run!(sink, logging);
    let elapsed = now.elapsed().as_millis();

    thread::sleep(Duration::from_secs(1));
    println!("Model run {}s in {}ms ()", sim_duration, elapsed);

    let tau = (sim_sampling_frequency as f64).recip();
    (0..6)
        .map(|k| {
            logging
                .deref()
                .iter()
                .flatten()
                .skip(k)
                .step_by(6)
                .cloned()
                .collect::<Vec<f64>>()
        })
        .enumerate()
        .for_each(|(k, rbm)| {
            let _: complot::Plot = (
                rbm.chunks(7).enumerate().map(|(i, x)| {
                    (
                        i as f64 * tau,
                        x.iter().map(|x| x * 1e6).collect::<Vec<f64>>(),
                    )
                }),
                complot::complot!(
                    format!("examples/figures/m1_rbm_ctrl-{}.png", k + 1),
                    xlabel = "Time [s]",
                    ylabel = ""
                ),
            )
                .into();
        });
    Ok(())
}
