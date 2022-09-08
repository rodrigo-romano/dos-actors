use dos_actors::{
    clients::{
        asm::*,
        m1::*,
        mount::{Mount, MountEncoders, MountSetPoint, MountTorques},
    },
    prelude::*,
};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use std::time::Instant;
mod config;
use config::Config;

#[tokio::test]
async fn setpoint_mount_m1_asm() -> anyhow::Result<()> {
    let config = Config::load()?;
    println!("{:?}", config);

    let sim_sampling_frequency = config.sampling;
    let sim_duration = 4_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(config.damping)
            .ins::<OSSElDriveTorque>()
            .ins::<OSSAzDriveTorque>()
            .ins::<OSSRotDriveTorque>()
            .ins::<OSSHarpointDeltaF>()
            .ins::<M1ActuatorsSegment1>()
            .ins::<M1ActuatorsSegment2>()
            .ins::<M1ActuatorsSegment3>()
            .ins::<M1ActuatorsSegment4>()
            .ins::<M1ActuatorsSegment5>()
            .ins::<M1ActuatorsSegment6>()
            .ins::<M1ActuatorsSegment7>()
            .ins::<MCM2SmHexF>()
            .ins::<MCM2CP6F>()
            .ins::<MCM2RB6F>()
            .ins::<MCM2Lcl6F>()
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSHardpointD>()
            .outs::<MCM2SmHexD>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .outs::<MCM2RB6D>()
            .use_static_gain_compensation(n_io)
            .build()?
    };

    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT
    let mut mount: Actor<_> = Mount::new().into();

    const M1_RATE: usize = 80;
    assert_eq!(sim_sampling_frequency / M1_RATE, 100);

    // HARDPOINTS
    let mut m1_hardpoints: Actor<_> = m1_ctrl::hp_dynamics::Controller::new().into();
    // LOADCELLS
    let mut m1_hp_loadcells: Actor<_, 1, M1_RATE> =
        m1_ctrl::hp_load_cells::Controller::new().into();
    // M1 SEGMENTS ACTUATORS
    let mut m1_segment1: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment1::Controller::new().into();
    let mut m1_segment2: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment2::Controller::new().into();
    let mut m1_segment3: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment3::Controller::new().into();
    let mut m1_segment4: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment4::Controller::new().into();
    let mut m1_segment5: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment5::Controller::new().into();
    let mut m1_segment6: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment6::Controller::new().into();
    let mut m1_segment7: Actor<_, M1_RATE, 1> =
        m1_ctrl::actuators::segment7::Controller::new().into();

    let logging = Logging::default().n_entry(2).into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let mut mount_set_point: Initiator<_> = Signals::new(3, n_step).into();
    mount_set_point
        .add_output()
        .build::<MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_output()
        .build::<MountTorques>()
        .into_input(&mut fem);

    let mut m1rbm_set_point: Initiator<_> = Signals::new(42, n_step).into();
    m1rbm_set_point
        .add_output()
        .build::<M1RBMcmd>()
        .into_input(&mut m1_hardpoints);
    m1_hardpoints
        .add_output()
        .multiplex(2)
        .build::<OSSHarpointDeltaF>()
        .into_input(&mut fem)
        .into_input(&mut m1_hp_loadcells);

    m1_hp_loadcells
        .add_output()
        .build::<S1HPLC>()
        .into_input(&mut m1_segment1);
    m1_hp_loadcells
        .add_output()
        .build::<S2HPLC>()
        .into_input(&mut m1_segment2);
    m1_hp_loadcells
        .add_output()
        .build::<S3HPLC>()
        .into_input(&mut m1_segment3);
    m1_hp_loadcells
        .add_output()
        .build::<S4HPLC>()
        .into_input(&mut m1_segment4);
    m1_hp_loadcells
        .add_output()
        .build::<S5HPLC>()
        .into_input(&mut m1_segment5);
    m1_hp_loadcells
        .add_output()
        .build::<S6HPLC>()
        .into_input(&mut m1_segment6);
    m1_hp_loadcells
        .add_output()
        .build::<S7HPLC>()
        .into_input(&mut m1_segment7);

    m1_segment1
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<M1ActuatorsSegment1>()
        .into_input(&mut fem);
    m1_segment2
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<M1ActuatorsSegment2>()
        .into_input(&mut fem);
    m1_segment3
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<M1ActuatorsSegment3>()
        .into_input(&mut fem);
    m1_segment4
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<M1ActuatorsSegment4>()
        .into_input(&mut fem);
    m1_segment5
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<M1ActuatorsSegment5>()
        .into_input(&mut fem);
    m1_segment6
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<M1ActuatorsSegment6>()
        .into_input(&mut fem);
    m1_segment7
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<M1ActuatorsSegment7>()
        .into_input(&mut fem);

    // M2 POSITIONER COMMAND
    let mut m2_pos_cmd: Initiator<_> = Signals::new(42, n_step).into();
    // FSM POSITIONNER
    let mut m2_positionner: Actor<_> = m2_ctrl::positionner::Controller::new().into();
    m2_pos_cmd
        .add_output()
        .build::<M2poscmd>()
        .into_input(&mut m2_positionner);
    m2_positionner
        .add_output()
        .build::<MCM2SmHexF>()
        .into_input(&mut fem);
    // ASM SET POINT
    let mut asm_cmd: Initiator<_> = (
        (0..7).fold(Signals::new(21, n_step), |s, i| {
            let a = (i + 1) as f64 * 0.25e-6;
            s.output_signal(
                i * 3,
                Signal::Sigmoid {
                    amplitude: a,
                    sampling_frequency_hz: sim_sampling_frequency as f64,
                },
            )
            .output_signal(
                i * 3 + 1,
                Signal::Sigmoid {
                    amplitude: -a,
                    sampling_frequency_hz: sim_sampling_frequency as f64,
                },
            )
            .output_signal(
                i * 3 + 2,
                Signal::Sigmoid {
                    amplitude: a,
                    sampling_frequency_hz: sim_sampling_frequency as f64,
                },
            )
        }),
        "ASMS Set Point",
    )
        .into();
    // ASM INNER CONTROLLER
    let mut asm_inner: Actor<_> = (
        m2_ctrl::ptt_fluid_damping::Controller::new(),
        "ASMS (piston,tip,tilt & fluid damping)",
    )
        .into();
    asm_cmd
        .add_output()
        .build::<Rrbfs>()
        .into_input(&mut asm_inner);
    asm_inner
        .add_output()
        .build::<MCM2CP6F>()
        .into_input(&mut fem);
    asm_inner
        .add_output()
        .build::<MCM2RB6F>()
        .into_input(&mut fem);
    asm_inner
        .add_output()
        .build::<MCM2Lcl6F>()
        .into_input(&mut fem);

    fem.add_output()
        .bootstrap()
        .build::<MountEncoders>()
        .into_input(&mut mount);
    fem.add_output()
        .bootstrap()
        .build::<OSSHardpointD>()
        .into_input(&mut m1_hp_loadcells);
    fem.add_output()
        .bootstrap()
        .build::<OSSM1Lcl>()
        .into_input(&mut sink);
    fem.add_output()
        .bootstrap()
        .build::<MCM2SmHexD>()
        .into_input(&mut m2_positionner);
    fem.add_output()
        .multiplex(2)
        .bootstrap()
        .build::<MCM2Lcl6D>()
        .into_input(&mut sink)
        .into_input(&mut asm_inner);
    fem.add_output()
        .bootstrap()
        .build::<MCM2RB6D>()
        .into_input(&mut asm_inner);

    let now = Instant::now();
    Model::new(vec![
        Box::new(mount_set_point),
        Box::new(mount),
        Box::new(m1rbm_set_point),
        Box::new(m1_hardpoints),
        Box::new(m1_hp_loadcells),
        Box::new(m1_segment1),
        Box::new(m1_segment2),
        Box::new(m1_segment3),
        Box::new(m1_segment4),
        Box::new(m1_segment5),
        Box::new(m1_segment6),
        Box::new(m1_segment7),
        Box::new(m2_pos_cmd),
        Box::new(m2_positionner),
        Box::new(asm_cmd),
        Box::new(asm_inner),
        Box::new(fem),
        Box::new(sink),
    ])
    .name("setpoint_mount-m1-asm")
    .flowchart()
    .check()?
    .run()
    .wait()
    .await?;
    println!("Elapsed time {}ms", now.elapsed().as_millis());

    println!("{}", *logging.lock().await);
    println!("M2 RBMS (x1e6):");
    (*logging.lock().await)
        .chunks()
        .last()
        .unwrap()
        .chunks(6)
        .skip(7)
        .for_each(|x| println!("{:+.3?}", x.iter().map(|x| x * 1e6).collect::<Vec<f64>>()));

    let rbm_residuals = (*logging.lock().await)
        .chunks()
        .last()
        .unwrap()
        .chunks(6)
        .skip(7)
        .enumerate()
        .map(|(i, x)| {
            x.iter()
                .skip(2)
                .take(3)
                .enumerate()
                .map(|(j, x)| x * 1e6 + (i + 1) as f64 * 0.25 * (-1f64).powi(j as i32))
                .map(|x| x * x)
                .sum::<f64>()
                / 6f64
        })
        .sum::<f64>()
        / 7f64;

    println!("M2 RBM set points RSS error: {}", rbm_residuals.sqrt());

    assert!(rbm_residuals.sqrt() < 1e-2);

    Ok(())
}
