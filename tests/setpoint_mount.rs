use dos_actors::clients::mount::{Mount, MountEncoders, MountSetPoint, MountTorques};
use dos_actors::{clients::arrow_client::Arrow, prelude::*};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use lom::{OpticalMetrics, LOM};
use skyangle::Conversion;
use std::env;

#[tokio::test]
async fn setpoint_mount() -> anyhow::Result<()> {
    setpoint_mount_at(None).await
}
#[tokio::test]
async fn setpoint_mount_00() -> anyhow::Result<()> {
    setpoint_mount_at(Some(0)).await
}
#[tokio::test]
async fn setpoint_mount_30() -> anyhow::Result<()> {
    setpoint_mount_at(Some(30)).await
}
#[tokio::test]
async fn setpoint_mount_60() -> anyhow::Result<()> {
    setpoint_mount_at(Some(60)).await
}

async fn setpoint_mount_at(ze: Option<i32>) -> anyhow::Result<()> {
    env::set_var(
        "FEM_REPO",
        if let Some(ze) = ze {
            format!("/fsx/MT_mount_zen_{ze:02}_m1HFN_FSM")
        } else {
            "/fsx/20220308_1335_MT_mount_zen_30_m1HFN_FSM".to_string()
        },
    );
    let sim_sampling_frequency = 1000;
    let sim_duration = 30_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;

        let n_io = (fem.n_inputs(), fem.n_outputs());
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .max_eigen_frequency(75f64)
            .ins::<OSSElDriveTorque>()
            .ins::<OSSAzDriveTorque>()
            .ins::<OSSRotDriveTorque>()
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .use_static_gain_compensation(n_io)
            .build()?
    };

    let mut source: Initiator<_> = Signals::new(3, n_step)
        .output_signal(0, Signal::Constant(1f64.from_arcsec()))
        .into();
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT
    let mut mount: Actor<_> = if let Some(ze) = ze {
        Mount::at_zenith_angle(ze)?
    } else {
        Mount::new()
    }
    .into();
    let logging = Arrow::builder(n_step).no_save().build().into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    source
        .add_output()
        .build::<MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_output()
        .build::<MountTorques>()
        .into_input(&mut fem);
    fem.add_output()
        .bootstrap()
        .build::<MountEncoders>()
        .into_input(&mut mount);
    fem.add_output()
        .unbounded()
        .build::<OSSM1Lcl>()
        .log(&mut sink)
        .await;
    fem.add_output()
        .unbounded()
        .build::<MCM2Lcl6D>()
        .log(&mut sink)
        .await;

    Model::new(vec![
        Box::new(source),
        Box::new(mount),
        Box::new(fem),
        Box::new(sink),
    ])
    .check()?
    .run()
    .wait()
    .await?;

    let lom = LOM::builder()
        .rigid_body_motions_record((*logging.lock().await).record()?)?
        .build()?;
    let segment_tiptilt = lom.segment_tiptilt();
    let stt = segment_tiptilt.items().last().unwrap();

    println!("Segment TT: {:.3?}mas", stt.to_mas());
    //assert!(tt[0].hypot(tt[1]) < 0.25);

    Ok(())
}
