//! M2 fast tip-tilt controller null test
//!
//! Run the mount, M1 force loop, M2 positioner, M2 piezostack and M2 fast tip-tilt feedback loop controllers with the FEM model
//! and with the set points of all the controllers set to 0
//! The optical model uses a geometric 24x24 Shack-Hartmann WFS,
//! The FEM model repository is read from the `FEM_REPO` environment variable

use crseo::{calibrations, Builder, Calibration, FromBuilder, Source, SH24 as TT7};
use dos_actors::{
    clients::{
        arrow_client::Arrow,
        ceo,
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
use lom::{Loader, LoaderTrait, OpticalMetrics, OpticalSensitivities, OpticalSensitivity, LOM};
use nalgebra as na;
use std::time::Instant;

#[tokio::test]
async fn setpoint_mount_m1_m2_tt() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000;
    let sim_duration = 4_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;
        let n_io = (fem.n_inputs(), fem.n_outputs());
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            //.truncate_hankel_singular_values(1e-4)
            .max_eigen_frequency(75.)
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
    let mut mount: Actor<_> = Mount::at_zenith_angle(30)?.into();

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

    let logging = Arrow::builder(n_step)
        .filename("setpoint-mount-m1-m2-tt.parquet")
        .no_save()
        .build()
        .into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    let mut mount_set_point: Initiator<_> = (Signals::new(3, n_step), "Mount_setpoint").into();
    mount_set_point
        .add_output()
        .build::<MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_output()
        .build::<MountTorques>()
        .into_input(&mut fem);

    let mut m1rbm_set_point: Initiator<_> = (Signals::new(42, n_step), "M1_RBM").into();
    m1rbm_set_point
        .add_output()
        .build::<M1RBMcmd>()
        .into_input(&mut m1_hardpoints);
    m1_hardpoints
        .add_output()
        .multiplex(2)
        .build::<OSSHarpointDeltaF>()
        .into_input(&mut fem)
        .into_input(&mut m1_hp_loadcells)
        .confirm()?;

    m1_hp_loadcells
        .add_output()
        .build::<S1HPLC>()
        .into_input(&mut m1_segment1)
        .confirm()?
        .add_output()
        .build::<S2HPLC>()
        .into_input(&mut m1_segment2)
        .confirm()?
        .add_output()
        .build::<S3HPLC>()
        .into_input(&mut m1_segment3)
        .confirm()?
        .add_output()
        .build::<S4HPLC>()
        .into_input(&mut m1_segment4)
        .confirm()?
        .add_output()
        .build::<S5HPLC>()
        .into_input(&mut m1_segment5)
        .confirm()?
        .add_output()
        .build::<S6HPLC>()
        .into_input(&mut m1_segment6)
        .confirm()?
        .add_output()
        .build::<S7HPLC>()
        .into_input(&mut m1_segment7)
        .confirm()?;

    m1_segment1
        .add_output()
        .bootstrap()
        .build::<M1ActuatorsSegment1>()
        .into_input(&mut fem);
    m1_segment2
        .add_output()
        .bootstrap()
        .build::<M1ActuatorsSegment2>()
        .into_input(&mut fem);
    m1_segment3
        .add_output()
        .bootstrap()
        .build::<M1ActuatorsSegment3>()
        .into_input(&mut fem);
    m1_segment4
        .add_output()
        .bootstrap()
        .build::<M1ActuatorsSegment4>()
        .into_input(&mut fem);
    m1_segment5
        .add_output()
        .bootstrap()
        .build::<M1ActuatorsSegment5>()
        .into_input(&mut fem);
    m1_segment6
        .add_output()
        .bootstrap()
        .build::<M1ActuatorsSegment6>()
        .into_input(&mut fem);
    m1_segment7
        .add_output()
        .bootstrap()
        .build::<M1ActuatorsSegment7>()
        .into_input(&mut fem);

    const FSM_RATE: usize = 5;
    assert_eq!(sim_sampling_frequency / FSM_RATE, 200);

    // M2 POSITIONER COMMAND
    let mut m2_pos_cmd: Initiator<_> = (Signals::new(42, n_step), "M2_pos").into();
    // FSM POSITIONNER
    let mut m2_positionner: Actor<_> = fsm::positionner::Controller::new().into();
    m2_pos_cmd
        .add_output()
        .build::<M2poscmd>()
        .into_input(&mut m2_positionner);
    m2_positionner
        .add_output()
        .build::<MCM2SmHexF>()
        .into_input(&mut fem);
    // FSM PIEZOSTACK
    let mut m2_piezostack: Actor<_> = fsm::piezostack::Controller::new().into();
    m2_piezostack
        .add_output()
        .build::<MCM2PZTF>()
        .into_input(&mut fem);
    // FSM TIP-TILT CONTROL
    let mut tiptilt_set_point: Initiator<_, FSM_RATE> = (
        (0..7).fold(Signals::new(14, n_step), |s, i| {
            (0..2).fold(s, |ss, j| {
                ss.output_signal(
                    i * 2 + j,
                    Signal::Constant((-1f64).powi((i + j) as i32) * 1e-6),
                )
            })
        }),
        "TipTilt_setpoint",
    )
        .into();
    let mut m2_tiptilt: Actor<_, FSM_RATE, 1> = fsm::tiptilt::Controller::new().into();
    tiptilt_set_point
        .add_output()
        .build::<TTSP>()
        .into_input(&mut m2_tiptilt);
    m2_tiptilt
        .add_output()
        .bootstrap()
        .build::<PZTcmd>()
        .into_input(&mut m2_piezostack);
    // OPTICAL MODEL (Geometric)
    let mut agws_tt7: Actor<_, 1, FSM_RATE> = {
        let mut agws_sh24 = ceo::OpticalModel::builder()
            .source(Source::builder().fwhm(8.0))
            .options(vec![ceo::OpticalModelOptions::ShackHartmann {
                options: ceo::ShackHartmannOptions::Geometric(*TT7::<crseo::Geometric>::new()),
                flux_threshold: 0.5,
            }])
            .build()?;
        use calibrations::Mirror;
        use calibrations::Segment::*;
        // GMT 2 WFS
        let mut gmt2wfs = Calibration::new(
            &agws_sh24.gmt,
            &agws_sh24.src,
            TT7::<crseo::Geometric>::new(),
        );
        let specs = vec![Some(vec![(Mirror::M2, vec![Rxyz(1e-6, Some(0..2))])]); 7];
        let now = Instant::now();
        gmt2wfs.calibrate(
            specs,
            calibrations::ValidLensletCriteria::OtherSensor(
                &mut agws_sh24.sensor.as_mut().unwrap(),
            ),
        );
        println!(
            "GMT 2 WFS calibration [{}x{}] in {}s",
            gmt2wfs.n_data,
            gmt2wfs.n_mode,
            now.elapsed().as_secs()
        );
        let dof_2_wfs: Vec<f64> = gmt2wfs.poke.into();
        let dof_2_wfs = na::DMatrix::<f64>::from_column_slice(
            dof_2_wfs.len() / gmt2wfs.n_mode,
            gmt2wfs.n_mode,
            &dof_2_wfs,
        );
        let wfs_2_rxy = dof_2_wfs.clone().pseudo_inverse(1e-12).unwrap();
        let senses: OpticalSensitivities = Loader::<OpticalSensitivities>::default().load()?;
        let rxy_2_stt = senses[OpticalSensitivity::SegmentTipTilt(Vec::new())].m2_rxy()?;
        // serde_pickle::to_writer(&mut std::fs::File::create("poke_mat.pkl")?,
        //         &(rxy_2_stt.as_slice(),dof_2_wfs.as_slice()),
        //         Default::default())?;
        agws_sh24.sensor_matrix_transform(rxy_2_stt * wfs_2_rxy);
        agws_sh24.into()
    };
    let mut tt_feedback: Terminator<_, FSM_RATE> = (
        Arrow::builder(n_step)
            .filename("tt_feedback.parquet")
            .no_save()
            .build(),
        "TT Feedback",
    )
        .into();
    agws_tt7
        .add_output()
        .multiplex(2)
        .build::<TTFB>()
        .into_input(&mut m2_tiptilt)
        .logn(&mut tt_feedback, 14)
        .await
        .confirm()?;

    fem.add_output()
        .bootstrap()
        .build::<MountEncoders>()
        .into_input(&mut mount)
        .confirm()?
        .add_output()
        .bootstrap()
        .build::<OSSHardpointD>()
        .into_input(&mut m1_hp_loadcells)
        .confirm()?
        .add_output()
        .multiplex(2)
        .build::<OSSM1Lcl>()
        .into_input(&mut agws_tt7)
        .logn(&mut sink, 42)
        .await
        .confirm()?
        .add_output()
        .multiplex(2)
        .build::<MCM2Lcl6D>()
        .into_input(&mut agws_tt7)
        .logn(&mut sink, 42)
        .await
        .confirm()?
        .add_output()
        .bootstrap()
        .build::<MCM2SmHexD>()
        .into_input(&mut m2_positionner)
        .confirm()?
        .add_output()
        .bootstrap()
        .build::<MCM2PZTD>()
        .into_input(&mut m2_piezostack);

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
        Box::new(m2_piezostack),
        Box::new(tiptilt_set_point),
        Box::new(m2_tiptilt),
        Box::new(agws_tt7),
        Box::new(fem),
        Box::new(sink),
        Box::new(tt_feedback),
    ])
    .name("mount-m1-m2-tt")
    .flowchart()
    .check()?
    .run()
    .wait()
    .await?;

    let lom = LOM::builder()
        .rigid_body_motions_record((*logging.lock().await).record()?)?
        .build()?;
    let segment_tiptilt = lom.segment_tiptilt();
    let stt: Vec<_> = segment_tiptilt
        .items()
        .last()
        .unwrap()
        .into_iter()
        .map(|x| x * 1e6)
        .collect();
    println!("STT (last).: {:.3?}", stt);

    let stt_residuals = stt
        .chunks(2)
        .enumerate()
        .map(|(i, x)| {
            x.iter()
                .enumerate()
                .map(|(j, x)| x - (-1f64).powi((i + j) as i32))
                .map(|x| x * x)
                .sum::<f64>()
                / 2f64
        })
        .sum::<f64>()
        / 7f64;
    println!(
        "Segment X pupil TT set points RSS error: {}",
        stt_residuals.sqrt()
    );

    assert!(stt_residuals.sqrt() < 1e-3);

    Ok(())
}
