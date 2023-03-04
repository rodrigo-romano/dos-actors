use dos_actors::{
    clients::{
        arrow_client::Arrow,
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
use rand_distr::{Distribution, StandardNormal};
use std::fs::File;

fn fig_2_mode(sid: usize) -> na::DMatrix<f64> {
    let fig_2_mode: Vec<f64> =
        bincode::deserialize_from(File::open(format!("m1s{sid}fig2mode.bin")).unwrap()).unwrap();
    if sid < 7 {
        na::DMatrix::from_vec(162, 602, fig_2_mode)
    } else {
        na::DMatrix::from_vec(151, 579, fig_2_mode).insert_rows(151, 11, 0f64)
    }
}

#[tokio::test]
async fn setpoint_mount_m1() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000;
    let sim_duration = 15_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    const SID: usize = 1;
    type M1SegmentxAxialD = M1Segment1AxialD;
    type M1CtrlSx<'a> = m1_ctrl::actuators::segment1::Controller<'a>;
    type SxSAoffsetFcmd = S1SAoffsetFcmd;
    type SxHPLC = S1HPLC;
    type M1ActuatorsSegmentx = M1ActuatorsSegment1;
    let (n_actuator, n_mode) = if SID == 7 { (306, 151) } else { (335, 162) };

    const M1_RATE: usize = 100;
    assert_eq!(sim_sampling_frequency / M1_RATE, 10);

    type D = Vec<f64>;

    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());
        print!("{fem}");
        {}
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .max_eigen_frequency(75f64)
            .ins::<OSSElDriveTorque>()
            .ins::<OSSAzDriveTorque>()
            .ins::<OSSRotDriveTorque>()
            .ins::<OSSHarpointDeltaF>()
            .ins::<M1ActuatorsSegmentx>()
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSHardpointD>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .outs::<M1SegmentxAxialD>()
            //.outs_with::<M1SegmentxAxialD>(fig_2_mode(SID))
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
    let mut m1_segment1: Actor<_, M1_RATE, 1> = M1CtrlSx::new().into();
    //let mut m1_segment1: Actor<_, M1_RATE, 1> = Sampler::new(vec![0f64; n_actuator]).into();

    //let logging = Logging::default().n_entry(2).into_arcx();
    let logging = Arrow::builder(n_step)
        .entry::<f64, OSSM1Lcl>(42)
        .entry::<f64, MCM2Lcl6D>(42)
        .entry::<f64, M1SegmentxAxialD>(n_mode)
        .entry::<f64, M1ActuatorsSegmentx>(n_actuator)
        .entry::<f64, OSSHardpointD>(84)
        .entry::<f64, MountEncoders>(14)
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
       .into();
       let mut mode_m1s = vec![
           {
               let mut mode = vec![0f64; 162];
               mode[26] = 0e-6;
               na::DVector::from_vec(mode)
           };
           6
       ];
       mode_m1s.push({
           let mut mode = vec![0f64; 151];
           mode[26] = 0e-6;
           na::DVector::from_vec(mode)
       });
       // M1S1 -------------------------------------------------------------------------------
       let mode_2_force = {
           let mode_2_force: Vec<f64> =
               bincode::deserialize_from(File::open(format!("m1s{SID}mode2forces.bin")).unwrap())
                   .unwrap();
           println!("{}", mode_2_force.len());
           na::DMatrix::from_vec(n_actuator, n_mode, mode_2_force)
       };
       let m1s1_force = mode_2_force * &mode_m1s[SID - 1];
    */
    let mut rng = rand::thread_rng();
    let m1sx: Vec<f64> = StandardNormal
        .sample_iter(&mut rng)
        .take(n_actuator)
        .collect();
    let mut m1s1f_set_point: Initiator<_, M1_RATE> = (
        Into::<Signals>::into((m1sx, n_step)),
        format!("M1S{SID}_setpoint"),
    )
        .into();

    m1s1f_set_point
        .add_output()
        .build::<D, SxSAoffsetFcmd>()
        .into_input(&mut m1_segment1);

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
        .build::<D, SxHPLC>()
        .into_input(&mut m1_segment1);

    m1_segment1
        .add_output()
        .multiplex(2)
        .bootstrap()
        .build::<D, M1ActuatorsSegmentx>()
        .into_input(&mut fem)
        .into_input(&mut sink);

    fem.add_output()
        .bootstrap()
        .multiplex(2)
        .build::<D, MountEncoders>()
        .into_input(&mut mount)
        .into_input(&mut sink);
    fem.add_output()
        .multiplex(2)
        .bootstrap()
        .build::<D, OSSHardpointD>()
        .into_input(&mut m1_hp_loadcells)
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, OSSM1Lcl>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, MCM2Lcl6D>()
        .into_input(&mut sink);
    fem.add_output()
        .build::<D, M1SegmentxAxialD>()
        .into_input(&mut sink);

    Model::new(vec![
        Box::new(mount_set_point),
        Box::new(mount),
        Box::new(m1s1f_set_point),
        Box::new(m1rbm_set_point),
        Box::new(m1_hardpoints),
        Box::new(m1_hp_loadcells),
        Box::new(m1_segment1),
        Box::new(fem),
        Box::new(sink),
    ])
    .name("mount-m1sx")
    .flowchart()
    .check()?
    .run()
    .wait()
    .await?;

    /*
        let m1sifig = (*logging.lock().await)
            .get(format!("M1Segment{SID}AxialD"))
            .unwrap();
        let mode_from_fig = na::DVector::from_column_slice(m1sifig.last().as_ref().unwrap().as_slice());
        //println!("{:.3}", mode_from_fig.map(|x| x * 1e6));
        let mode_err = (&mode_m1s[SID - 1] - mode_from_fig).norm();
        println!(
            "M1S{} mode vector estimate error (x10e6): {:.3}",
            SID,
            mode_err * 1e6
        );
    */

    Ok(())
}
