use dos_actors::{
    clients::{
        fsm::*,
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

#[tokio::test]
async fn setpoint_mount_m1_m2() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000;
    let sim_duration = 4_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
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
            .ins::<MCM2PZTF>()
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSHardpointD>()
            .outs::<MCM2SmHexD>()
            .outs::<MCM2PZTD>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .use_static_gain_compensation(n_io)
            .build()?
    };

    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT
    let mut mount: Actor<_> = Mount::new().into();

    const M1_RATE: usize = 10;
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

    type D = Vec<f64>;

    let mut mount_set_point: Initiator<_> = Signals::new(3, n_step).into();
    mount_set_point
        .add_output()
        .build::<D, MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_output()
        .build::<D, MountTorques>()
        .into_input(&mut fem);

    let mut m1rbm_set_point: Initiator<_> = Signals::new(42, n_step).into();
    m1rbm_set_point
        .add_output()
        .build::<D, M1RBMcmd>()
        .into_input(&mut m1_hardpoints);
    m1_hardpoints
        .add_output()
        .multiplex(2)
        .build::<D, OSSHarpointDeltaF>()
        .into_input(&mut fem)
        .into_input(&mut m1_hp_loadcells);

    m1_hp_loadcells
        .add_output()
        .build::<D, S1HPLC>()
        .into_input(&mut m1_segment1);
    m1_hp_loadcells
        .add_output()
        .build::<D, S2HPLC>()
        .into_input(&mut m1_segment2);
    m1_hp_loadcells
        .add_output()
        .build::<D, S3HPLC>()
        .into_input(&mut m1_segment3);
    m1_hp_loadcells
        .add_output()
        .build::<D, S4HPLC>()
        .into_input(&mut m1_segment4);
    m1_hp_loadcells
        .add_output()
        .build::<D, S5HPLC>()
        .into_input(&mut m1_segment5);
    m1_hp_loadcells
        .add_output()
        .build::<D, S6HPLC>()
        .into_input(&mut m1_segment6);
    m1_hp_loadcells
        .add_output()
        .build::<D, S7HPLC>()
        .into_input(&mut m1_segment7);

    m1_segment1
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<D, M1ActuatorsSegment1>()
        .into_input(&mut fem);
    m1_segment2
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<D, M1ActuatorsSegment2>()
        .into_input(&mut fem);
    m1_segment3
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<D, M1ActuatorsSegment3>()
        .into_input(&mut fem);
    m1_segment4
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<D, M1ActuatorsSegment4>()
        .into_input(&mut fem);
    m1_segment5
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<D, M1ActuatorsSegment5>()
        .into_input(&mut fem);
    m1_segment6
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<D, M1ActuatorsSegment6>()
        .into_input(&mut fem);
    m1_segment7
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<D, M1ActuatorsSegment7>()
        .into_input(&mut fem);

    // M2 POSITIONER COMMAND
    let mut m2_pos_cmd: Initiator<_> = (0..7)
        .fold(Signals::new(42, n_step), |s, i| {
            (0..6).fold(s, |ss, j| {
                ss.output_signal(
                    i * 6 + j,
                    Signal::Constant((-1f64).powi((i + j) as i32) * 1e-6),
                )
            })
        })
        .into();
    // FSM POSITIONNER
    let mut m2_positionner: Actor<_> = fsm::positionner::Controller::new().into();
    m2_pos_cmd
        .add_output()
        .build::<D, M2poscmd>()
        .into_input(&mut m2_positionner);
    m2_positionner
        .add_output()
        .build::<D, MCM2SmHexF>()
        .into_input(&mut fem);
    // FSM PIEZOSTACK COMMAND
    let mut m2_pzt_cmd: Initiator<_> = Signals::new(21, n_step).into();
    // FSM PIEZOSTACK
    let mut m2_piezostack: Actor<_> = fsm::piezostack::Controller::new().into();
    m2_pzt_cmd
        .add_output()
        .build::<D, PZTcmd>()
        .into_input(&mut m2_piezostack);
    m2_piezostack
        .add_output()
        .build::<D, MCM2PZTF>()
        .into_input(&mut fem);

    fem.add_output()
        .bootstrap()
        .unbounded()
        .build::<D, MountEncoders>()
        .into_input(&mut mount);
    fem.add_output()
        .bootstrap()
        .unbounded()
        .build::<D, OSSHardpointD>()
        .into_input(&mut m1_hp_loadcells);
    fem.add_output()
        .unbounded()
        .build::<D, OSSM1Lcl>()
        .into_input(&mut sink);
    fem.add_output()
        .unbounded()
        .build::<D, MCM2Lcl6D>()
        .into_input(&mut sink);
    fem.add_output()
        .bootstrap()
        .unbounded()
        .build::<D, MCM2SmHexD>()
        .into_input(&mut m2_positionner);
    fem.add_output()
        .bootstrap()
        .unbounded()
        .build::<D, MCM2PZTD>()
        .into_input(&mut m2_piezostack);

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
        Box::new(m2_pzt_cmd),
        Box::new(m2_piezostack),
        Box::new(fem),
        Box::new(sink),
    ])
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
                .enumerate()
                .map(|(j, x)| x * 1e6 - (-1f64).powi((i + j) as i32))
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
