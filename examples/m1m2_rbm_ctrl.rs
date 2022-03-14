use std::time::Instant;

use dos_actors::clients::{
    fsm::*,
    m1::*,
    mount::{Mount, MountEncoders, MountTorques},
};
use dos_actors::prelude::*;
use fem::{
    dos::{DiscreteModalSolver, Exponential, ExponentialMatrix},
    fem_io::*,
    FEM,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //simple_logger::SimpleLogger::new().env().init().unwrap();

    let sim_sampling_frequency = 1000;
    let sim_duration = 4_usize;

    // FEM
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
    println!("{}", state_space);

    const M1_RATE: usize = 10;
    assert_eq!(sim_sampling_frequency / M1_RATE, 100);

    let n_segment = 7;
    let n_step = sim_sampling_frequency * sim_duration;
    let signals = (0..n_segment).fold(Signals::new(vec![42], n_step), |s, i| {
        (0..6).fold(s, |ss, j| {
            ss.output_signal(
                0,
                i * 6 + j,
                Signal::Constant((-1f64).powi(j as i32) * (1 + i) as f64 * 1e-6),
            )
        })
    });
    let mut source: Initiator<_> = signals.clone().into();
    // M2 POSITIONER COMMAND
    let mut m2_pos_cmd: Initiator<_> = signals.into();
    // M2 TT COMMAND
    let mut m2_tt_cmd: Initiator<_> = Signals::new(vec![21], n_step).into();
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT
    let mut mount: Actor<_> = Mount::new().into();
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
    // FSM POSITIONNER
    let mut m2_positionner: Actor<_> = fsm::positionner::Controller::new().into();
    // FSM PIEZOSTACK
    let mut m2_piezostack: Actor<_> = fsm::piezostack::Controller::new().into();

    type D = Vec<f64>;
    source
        .add_output::<D, M1RBMcmd>(None)
        .into_input(&mut &mut m1_hardpoints);

    m1_hardpoints
        .add_output::<D, OSSHarpointDeltaF>(Some(vec![1, 1]))
        .into_input(&mut fem)
        .into_input(&mut m1_hp_loadcells);

    mount
        .add_output::<D, MountTorques>(None)
        .into_input(&mut fem);

    m1_hp_loadcells
        .add_output::<D, S1HPLC>(None)
        .into_input(&mut m1_segment1);
    m1_hp_loadcells
        .add_output::<D, S2HPLC>(None)
        .into_input(&mut m1_segment2);
    m1_hp_loadcells
        .add_output::<D, S3HPLC>(None)
        .into_input(&mut m1_segment3);
    m1_hp_loadcells
        .add_output::<D, S4HPLC>(None)
        .into_input(&mut m1_segment4);
    m1_hp_loadcells
        .add_output::<D, S5HPLC>(None)
        .into_input(&mut m1_segment5);
    m1_hp_loadcells
        .add_output::<D, S6HPLC>(None)
        .into_input(&mut m1_segment6);
    m1_hp_loadcells
        .add_output::<D, S7HPLC>(None)
        .into_input(&mut m1_segment7);

    m1_segment1
        .add_output::<D, M1ActuatorsSegment1>(None)
        .into_input(&mut fem);
    m1_segment2
        .add_output::<D, M1ActuatorsSegment2>(None)
        .into_input(&mut fem);
    m1_segment3
        .add_output::<D, M1ActuatorsSegment3>(None)
        .into_input(&mut fem);
    m1_segment4
        .add_output::<D, M1ActuatorsSegment4>(None)
        .into_input(&mut fem);
    m1_segment5
        .add_output::<D, M1ActuatorsSegment5>(None)
        .into_input(&mut fem);
    m1_segment6
        .add_output::<D, M1ActuatorsSegment6>(None)
        .into_input(&mut fem);
    m1_segment7
        .add_output::<D, M1ActuatorsSegment7>(None)
        .into_input(&mut fem);

    m2_pos_cmd
        .add_output::<D, M2poscmd>(None)
        .into_input(&mut m2_positionner);
    m2_positionner
        .add_output::<D, MCM2SmHexF>(None)
        .into_input(&mut fem);

    m2_tt_cmd
        .add_output::<D, TTcmd>(None)
        .into_input(&mut m2_piezostack);
    m2_piezostack
        .add_output::<D, MCM2PZTF>(None)
        .into_input(&mut fem);

    let logging = Logging::<f64>::default().into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let m2_rbm = Logging::<f64>::default().into_arcx();
    let mut m2_rbm_logs = Terminator::<_>::new(m2_rbm.clone());

    fem.add_output::<D, OSSHardpointD>(None)
        .into_input(&mut m1_hp_loadcells);
    fem.add_output::<D, MountEncoders>(None)
        .into_input(&mut mount);
    fem.add_output::<D, MCM2SmHexD>(None)
        .into_input(&mut m2_positionner);
    fem.add_output::<D, MCM2PZTD>(None)
        .into_input(&mut m2_piezostack);
    fem.add_output::<D, OSSM1Lcl>(None).into_input(&mut sink);
    fem.add_output::<D, MCM2Lcl6D>(None)
        .into_input(&mut m2_rbm_logs);

    println!("{source}{m1_hardpoints}{m1_hp_loadcells}{m2_positionner}{fem}");

    source.spawn();
    mount.spawn();
    m1_hardpoints.spawn();
    m1_hp_loadcells.spawn();
    m2_pos_cmd.spawn();
    m2_positionner.spawn();
    m2_tt_cmd.spawn();
    m2_piezostack.spawn();
    m2_rbm_logs.spawn();

    spawn_bootstrap!(
        m1_segment1::<D, M1ActuatorsSegment1>,
        m1_segment2::<D, M1ActuatorsSegment2>,
        m1_segment3::<D, M1ActuatorsSegment3>,
        m1_segment4::<D, M1ActuatorsSegment4>,
        m1_segment5::<D, M1ActuatorsSegment5>,
        m1_segment6::<D, M1ActuatorsSegment6>,
        m1_segment7::<D, M1ActuatorsSegment7>
    );

    fem.bootstrap::<D, MountEncoders>()
        .await
        .bootstrap::<D, OSSHardpointD>()
        .await
        .bootstrap::<D, MCM2SmHexD>()
        .await
        .bootstrap::<D, MCM2PZTD>()
        .await;
    fem.spawn();

    println!("Starting the model");
    let now = Instant::now();

    sink.run().await;
    let elapsed = now.elapsed().as_millis();

    println!("Model run {}s in {}ms ()", sim_duration, elapsed);

    let tau = (sim_sampling_frequency as f64).recip();
    let labels = vec!["Tx", "Ty", "Tz", "Rx", "Ry", "Rz"];

    {
        let logging_lock = logging.lock().await;
        (0..6)
            .map(|k| {
                (**logging_lock)
                    .iter()
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
                        ylabel = labels[k]
                    ),
                )
                    .into();
            });
    }

    {
        let logging_lock = m2_rbm.lock().await;
        (0..6)
            .map(|k| {
                (**logging_lock)
                    .iter()
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
                        format!("examples/figures/m2_rbm_ctrl-{}.png", k + 1),
                        xlabel = "Time [s]",
                        ylabel = labels[k]
                    ),
                )
                    .into();
            });
    }
    Ok(())
}
