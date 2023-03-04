use dos_actors::{
    clients::{
        arrow_client::Arrow,
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
use nalgebra as na;
use rand::Rng;
use std::fs::File;

fn fig_2_mode(sid: u32) -> na::DMatrix<f64> {
    let fig_2_mode: Vec<f64> =
        bincode::deserialize_from(File::open(format!("m1s{sid}fig2mode.bin")).unwrap()).unwrap();
    if sid < 7 {
        na::DMatrix::from_vec(162, 602, fig_2_mode)
    } else {
        na::DMatrix::from_vec(151, 579, fig_2_mode)
    }
}

#[tokio::test]
async fn setpoint_mount_m1() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000;
    let sim_duration = 4_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    const M1_RATE: usize = 10;
    assert_eq!(sim_sampling_frequency / M1_RATE, 100);

    type D = Vec<f64>;

    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());
        print!("{fem}");
        {}
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
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSHardpointD>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .outs_with::<M1Segment1AxialD>(fig_2_mode(1))
            .outs_with::<M1Segment2AxialD>(fig_2_mode(2))
            .outs_with::<M1Segment3AxialD>(fig_2_mode(3))
            .outs_with::<M1Segment4AxialD>(fig_2_mode(4))
            .outs_with::<M1Segment5AxialD>(fig_2_mode(5))
            .outs_with::<M1Segment6AxialD>(fig_2_mode(6))
            .outs_with::<M1Segment7AxialD>(fig_2_mode(7))
            .use_static_gain_compensation(n_io)
            .build()?
    };
    /*
        {
            for i in 0..7 {
                let mut fem = FEM::from_env()?.static_from_env()?;
                let n_io = (fem.n_inputs(), fem.n_outputs());
                let nodes = fem.outputs[i + 1]
                    .as_ref()
                    .unwrap()
                    .get_by(|x| x.properties.location.as_ref().map(|x| x.to_vec()))
                    .into_iter()
                    .flatten()
                    .collect::<Vec<f64>>();
                serde_pickle::to_writer(
                    &mut File::create(format!("M1Segment{}AxialD.pkl", i + 1))?,
                    &nodes,
                    Default::default(),
                )?;
                serde_pickle::to_writer(
                    &mut File::create(format!("m1s{}f2d.pkl", i + 1))?,
                    &fem.keep_inputs(&[i + 1])
                        .keep_outputs(&[i + 1])
                        .reduced_static_gain(n_io)
                        .unwrap()
                        .as_slice()
                        .to_vec(),
                    Default::default(),
                )?;
            }
        }
    */
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

    //let logging = Logging::default().n_entry(2).into_arcx();
    let logging = Arrow::builder(n_step)
        .entry::<f64, OSSM1Lcl>(42)
        .entry::<f64, MCM2Lcl6D>(42)
        .entry::<f64, M1Segment1AxialD>(162)
        .entry::<f64, M1Segment2AxialD>(162)
        .entry::<f64, M1Segment3AxialD>(162)
        .entry::<f64, M1Segment4AxialD>(162)
        .entry::<f64, M1Segment5AxialD>(162)
        .entry::<f64, M1Segment6AxialD>(162)
        .entry::<f64, M1Segment7AxialD>(151)
        .build()
        .into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let mut mount_set_point: Initiator<_> = (Signals::new(3, n_step), "Mount_setpoint").into();
    mount_set_point
        .add_output()
        .build::<D, MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_output()
        .build::<D, MountTorques>()
        .into_input(&mut fem);

    /*let m1s1f_set_point: Initiator<_, M1_RATE> = Signals::new(335, n_step)
        .output_signal(0, Signal::Constant(100f64))
        .into();
    let mut m1s1f_set_point: Initiator<_, M1_RATE> = (0..335)
        .step_by(5)
        .fold(Signals::new(335, n_step), |s, i| {
            s.output_signal(
                i,
                Signal::Constant(rand::thread_rng().gen_range(-100f64..100f64)),
            )
        })
    .into();*/
    let mut mode_m1s = vec![
        {
            let mut mode = vec![0f64; 162];
            mode[26] = 1e-6;
            na::DVector::from_vec(mode)
        };
        6
    ];
    mode_m1s.push({
        let mut mode = vec![0f64; 151];
        mode[26] = 1e-6;
        na::DVector::from_vec(mode)
    });
    // M1S1 -------------------------------------------------------------------------------
    let mode_2_force = {
        let mode_2_force: Vec<f64> =
            bincode::deserialize_from(File::open("m1s1mode2forces.bin").unwrap()).unwrap();
        println!("{}", mode_2_force.len());
        na::DMatrix::from_vec(335, 162, mode_2_force)
    };
    let m1s1_force = mode_2_force * &mode_m1s[0];
    let mut m1s1f_set_point: Initiator<_, M1_RATE> = (
        m1s1_force
            .as_slice()
            .iter()
            .enumerate()
            .fold(Signals::new(335, n_step), |s, (i, v)| {
                s.output_signal(i, Signal::Constant(*v))
            }),
        "M1S1_setpoint",
    )
        .into();
    m1s1f_set_point
        .add_output()
        .build::<D, S1SAoffsetFcmd>()
        .into_input(&mut m1_segment1);
    // M1S2 -------------------------------------------------------------------------------
    let mode_2_force = {
        let mode_2_force: Vec<f64> =
            bincode::deserialize_from(File::open("m1s2mode2forces.bin").unwrap()).unwrap();
        println!("{}", mode_2_force.len());
        na::DMatrix::from_vec(335, 162, mode_2_force)
    };
    let m1s2_force = mode_2_force * &mode_m1s[1];
    let mut m1s2f_set_point: Initiator<_, M1_RATE> = (
        m1s2_force
            .as_slice()
            .iter()
            .enumerate()
            .fold(Signals::new(335, n_step), |s, (i, v)| {
                s.output_signal(i, Signal::Constant(*v))
            }),
        "M1S2_setpoint",
    )
        .into();
    m1s2f_set_point
        .add_output()
        .build::<D, S2SAoffsetFcmd>()
        .into_input(&mut m1_segment2);
    // M1S3 -------------------------------------------------------------------------------
    let mode_2_force = {
        let mode_2_force: Vec<f64> =
            bincode::deserialize_from(File::open("m1s3mode2forces.bin").unwrap()).unwrap();
        println!("{}", mode_2_force.len());
        na::DMatrix::from_vec(335, 162, mode_2_force)
    };
    let m1s3_force = mode_2_force * &mode_m1s[2];
    let mut m1s3f_set_point: Initiator<_, M1_RATE> = (
        m1s3_force
            .as_slice()
            .iter()
            .enumerate()
            .fold(Signals::new(335, n_step), |s, (i, v)| {
                s.output_signal(i, Signal::Constant(*v))
            }),
        "M1S3_setpoint",
    )
        .into();
    m1s3f_set_point
        .add_output()
        .build::<D, S3SAoffsetFcmd>()
        .into_input(&mut m1_segment3);
    // M1S4 -------------------------------------------------------------------------------
    let mode_2_force = {
        let mode_2_force: Vec<f64> =
            bincode::deserialize_from(File::open("m1s4mode2forces.bin").unwrap()).unwrap();
        println!("{}", mode_2_force.len());
        na::DMatrix::from_vec(335, 162, mode_2_force)
    };
    let m1s4_force = mode_2_force * &mode_m1s[3];
    let mut m1s4f_set_point: Initiator<_, M1_RATE> = (
        m1s4_force
            .as_slice()
            .iter()
            .enumerate()
            .fold(Signals::new(335, n_step), |s, (i, v)| {
                s.output_signal(i, Signal::Constant(*v))
            }),
        "M1S4_setpoint",
    )
        .into();
    m1s4f_set_point
        .add_output()
        .build::<D, S4SAoffsetFcmd>()
        .into_input(&mut m1_segment4);
    // M1S5 -------------------------------------------------------------------------------
    let mode_2_force = {
        let mode_2_force: Vec<f64> =
            bincode::deserialize_from(File::open("m1s5mode2forces.bin").unwrap()).unwrap();
        println!("{}", mode_2_force.len());
        na::DMatrix::from_vec(335, 162, mode_2_force)
    };
    let m1s5_force = mode_2_force * &mode_m1s[4];
    let mut m1s5f_set_point: Initiator<_, M1_RATE> = (
        m1s5_force
            .as_slice()
            .iter()
            .enumerate()
            .fold(Signals::new(335, n_step), |s, (i, v)| {
                s.output_signal(i, Signal::Constant(*v))
            }),
        "M1S5_setpoint",
    )
        .into();
    m1s5f_set_point
        .add_output()
        .build::<D, S5SAoffsetFcmd>()
        .into_input(&mut m1_segment5);
    // M1S6 -------------------------------------------------------------------------------
    let mode_2_force = {
        let mode_2_force: Vec<f64> =
            bincode::deserialize_from(File::open("m1s6mode2forces.bin").unwrap()).unwrap();
        println!("{}", mode_2_force.len());
        na::DMatrix::from_vec(335, 162, mode_2_force)
    };
    let m1s6_force = mode_2_force * &mode_m1s[5];
    let mut m1s6f_set_point: Initiator<_, M1_RATE> = (
        m1s6_force
            .as_slice()
            .iter()
            .enumerate()
            .fold(Signals::new(335, n_step), |s, (i, v)| {
                s.output_signal(i, Signal::Constant(*v))
            }),
        "M1S6_setpoint",
    )
        .into();
    m1s6f_set_point
        .add_output()
        .build::<D, S6SAoffsetFcmd>()
        .into_input(&mut m1_segment6);
    // M1S7 -------------------------------------------------------------------------------
    let mode_2_force = {
        let mode_2_force: Vec<f64> =
            bincode::deserialize_from(File::open("m1s7mode2forces.bin").unwrap()).unwrap();
        println!("{}", mode_2_force.len());
        na::DMatrix::from_vec(306, 151, mode_2_force)
    };
    let m1s7_force = mode_2_force * &mode_m1s[6];
    let mut m1s7f_set_point: Initiator<_, M1_RATE> = (
        m1s7_force
            .as_slice()
            .iter()
            .enumerate()
            .fold(Signals::new(306, n_step), |s, (i, v)| {
                s.output_signal(i, Signal::Constant(*v))
            }),
        "M1S7_setpoint",
    )
        .into();
    m1s7f_set_point
        .add_output()
        .build::<D, S7SAoffsetFcmd>()
        .into_input(&mut m1_segment7);

    let mut m1rbm_set_point: Initiator<_> = (Signals::new(42, n_step), "M1RBM_setpoint").into();
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
        .build::<D, M1ActuatorsSegment1>()
        .into_input(&mut fem);
    m1_segment2
        .add_output()
        .bootstrap()
        .build::<D, M1ActuatorsSegment2>()
        .into_input(&mut fem);
    m1_segment3
        .add_output()
        .bootstrap()
        .build::<D, M1ActuatorsSegment3>()
        .into_input(&mut fem);
    m1_segment4
        .add_output()
        .bootstrap()
        .build::<D, M1ActuatorsSegment4>()
        .into_input(&mut fem);
    m1_segment5
        .add_output()
        .bootstrap()
        .build::<D, M1ActuatorsSegment5>()
        .into_input(&mut fem);
    m1_segment6
        .add_output()
        .bootstrap()
        .build::<D, M1ActuatorsSegment6>()
        .into_input(&mut fem);
    m1_segment7
        .add_output()
        .bootstrap()
        .build::<D, M1ActuatorsSegment7>()
        .into_input(&mut fem);

    fem.add_output()
        .bootstrap()
        .build::<D, MountEncoders>()
        .into_input(&mut mount);
    fem.add_output()
        .bootstrap()
        .build::<D, OSSHardpointD>()
        .into_input(&mut m1_hp_loadcells);
    fem.add_output()
        .build::<D, OSSM1Lcl>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, MCM2Lcl6D>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, M1Segment1AxialD>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, M1Segment2AxialD>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, M1Segment3AxialD>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, M1Segment4AxialD>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, M1Segment5AxialD>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, M1Segment6AxialD>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, M1Segment7AxialD>()
        .into_input(&mut sink);

    Model::new(vec![
        Box::new(mount_set_point),
        Box::new(mount),
        Box::new(m1s1f_set_point),
        Box::new(m1s2f_set_point),
        Box::new(m1s3f_set_point),
        Box::new(m1s4f_set_point),
        Box::new(m1s5f_set_point),
        Box::new(m1s6f_set_point),
        Box::new(m1s7f_set_point),
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
        Box::new(fem),
        Box::new(sink),
    ])
    .name("mount-m1")
    .flowchart()
    .check()?
    .run()
    .wait()
    .await?;

    for sid in 1..=7 {
        /*
        let fig_2_mode = {
            let fig_2_mode: Vec<f64> =
                bincode::deserialize_from(File::open(format!("m1s{sid}fig2mode.bin")).unwrap())
                    .unwrap();
            if sid < 7 {
                na::DMatrix::from_vec(162, 602, fig_2_mode)
            } else {
                na::DMatrix::from_vec(151, 579, fig_2_mode)
            }
        };*/
        let m1sifig = (*logging.lock().await)
            .get(format!("M1Segment{sid}AxialD"))
            .unwrap();
        let mode_from_fig =
            na::DVector::from_column_slice(m1sifig.last().as_ref().unwrap().as_slice());
        //println!("{:.3}", mode_from_fig.map(|x| x * 1e6));
        let mode_err = (&mode_m1s[sid - 1] - mode_from_fig).norm();
        println!(
            "M1S{} mode vector estimate error (x10e6): {:.3}",
            sid,
            mode_err * 1e6
        );
    }

    /*
    println!("{}", *logging.lock().await);
    println!("M1 RBMS (x1e6):");
    (*logging.lock().await)
        .chunks()
        .last()
        .unwrap()
        .chunks(6)
        .take(7)
        .for_each(|x| println!("{:+.3?}", x.iter().map(|x| x * 1e6).collect::<Vec<f64>>()));

    let rbm_residuals = (*logging.lock().await)
        .chunks()
        .last()
        .unwrap()
        .chunks(6)
        .take(7)
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

    println!("M1 RBM set points RSS error: {}", rbm_residuals.sqrt());

    assert!(rbm_residuals.sqrt() < 1e-2);
     */

    Ok(())
}
