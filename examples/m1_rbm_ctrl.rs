use dos_actors::prelude::*;
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
    let sim_duration = 30_usize;

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
    let mut m1_hardpoints = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("M1 hardpoints");
    let mut m1_hp_loadcells =
        Actor::<Vec<f64>, Vec<f64>, 1, M1_RATE>::new().tag("M1 hardpoints load cells");
    let mut fem = Actor::<Vec<f64>, Vec<f64>, 1, 1>::new().tag("FEM");
    let mut sink = Terminator::<Vec<f64>, 1>::build().tag("sink");

    let n_segment = 7;
    let (mut m1sx, mut sx_bm_cmd): (Vec<_>, Vec<_>) = (1..=n_segment)
        .map(|sid| {
            (
                Actor::<Vec<f64>, Vec<f64>, M1_RATE, 1>::new().tag(format!("M1 S{sid}")),
                Initiator::<Vec<f64>, M1_RATE>::build().tag(format!("BM S{sid}")),
            )
        })
        .unzip();

    channel![mount_controller => mount_driver];
    channel![mount_driver => fem; 3];

    channel![fem => sink];
    channel![fem => (mount_controller,mount_driver); 3];

    channel![rbm_cmd => m1_hardpoints];
    channel![fem => m1_hp_loadcells];
    channel![m1_hardpoints => (fem, m1_hp_loadcells)];

    dos_actors::one_to_many(
        &mut m1_hp_loadcells,
        m1sx.iter_mut()
            .collect::<Vec<&mut Actor<Vec<f64>, Vec<f64>, M1_RATE, 1>>>()
            .as_mut_slice(),
    );
    m1sx.iter_mut()
        .zip(sx_bm_cmd.iter_mut())
        .for_each(|(m1si, si_bm_cmd)| {
            dos_actors::one_to_many(si_bm_cmd, &mut [m1si]);
            dos_actors::one_to_many(m1si, &mut [&mut fem]);
        });

    let n_iterations = sim_sampling_frequency * sim_duration;
    let mut signals = (0..n_segment).fold(Signals::new(vec![42], n_iterations), |s, i| {
        (0..6).fold(s, |ss, j| {
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
    for (i, (mut si_bm_cmd, mut m1si)) in sx_bm_cmd.into_iter().zip(m1sx.into_iter()).enumerate() {
        match i + 1 {
            1 => {
                spawn!(
                    (si_bm_cmd, Signals::new(vec![27], n_iterations),),
                    (
                        m1si,
                        m1::actuators::segment1::Controller::new(),
                        vec![vec![0f64; 335]]
                    )
                );
            }
            2 => {
                spawn!(
                    (si_bm_cmd, Signals::new(vec![27], n_iterations),),
                    (
                        m1si,
                        m1::actuators::segment2::Controller::new(),
                        vec![vec![0f64; 335]]
                    )
                );
            }
            3 => {
                spawn!(
                    (si_bm_cmd, Signals::new(vec![27], n_iterations),),
                    (
                        m1si,
                        m1::actuators::segment3::Controller::new(),
                        vec![vec![0f64; 335]]
                    )
                );
            }
            4 => {
                spawn!(
                    (si_bm_cmd, Signals::new(vec![27], n_iterations),),
                    (
                        m1si,
                        m1::actuators::segment4::Controller::new(),
                        vec![vec![0f64; 335]]
                    )
                );
            }
            5 => {
                spawn!(
                    (si_bm_cmd, Signals::new(vec![27], n_iterations),),
                    (
                        m1si,
                        m1::actuators::segment5::Controller::new(),
                        vec![vec![0f64; 335]]
                    )
                );
            }
            6 => {
                spawn!(
                    (si_bm_cmd, Signals::new(vec![27], n_iterations),),
                    (
                        m1si,
                        m1::actuators::segment6::Controller::new(),
                        vec![vec![0f64; 335]]
                    )
                );
            }
            7 => {
                spawn!(
                    (si_bm_cmd, Signals::new(vec![27], n_iterations),),
                    (
                        m1si,
                        m1::actuators::segment7::Controller::new(),
                        vec![vec![0f64; 306]]
                    )
                );
            }
            _ => panic!("invalid segment #"),
        }
    }

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
