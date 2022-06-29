//! Mount controller null test
//!
//! Run the mount controller with the mount torques and encoders of the FEM model
//! and with the mount control set points set to 0
//! The FEM model repository is read from the `FEM_REPO` environment variable
//! The LOM sensitivity matrices are located in the directory given by the `LOM` environment variable

use dos_actors::clients::mount::{Mount, MountEncoders, MountSetPoint, MountTorques};
use dos_actors::{clients::arrow_client::Arrow, prelude::*};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use lom::{Stats, LOM};
use std::env;

#[tokio::test]
async fn zero_mount() -> anyhow::Result<()> {
    zero_mount_at(None).await
}
#[tokio::test]
async fn zero_mount_00() -> anyhow::Result<()> {
    zero_mount_at(Some(0)).await
}
#[tokio::test]
async fn zero_mount_30() -> anyhow::Result<()> {
    zero_mount_at(Some(30)).await
}
#[tokio::test]
async fn zero_mount_60() -> anyhow::Result<()> {
    zero_mount_at(Some(60)).await
}

async fn zero_mount_at(ze: Option<i32>) -> anyhow::Result<()> {
    env::set_var(
        "FEM_REPO",
        if let Some(ze) = ze {
            format!("/fsx/MT_mount_zen_{ze:02}_m1HFN_FSM")
        } else {
            "/fsx/20220308_1335_MT_mount_zen_30_m1HFN_FSM".to_string()
        },
    );

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
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .use_static_gain_compensation(n_io)
            .build()?
    };

    let mut source: Initiator<_> = Signals::new(3, n_step).into();
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
    .name("mount")
    .flowchart()
    .check()?
    .run()
    .wait()
    .await?;

    let lom = LOM::builder()
        .rigid_body_motions_record((*logging.lock().await).record()?)?
        .build()?;
    let tiptilt = lom.tiptilt_mas();
    let n_sample = 1000;
    let tt = tiptilt.std(Some(n_sample));
    println!("TT STD.: {:.3?}mas", tt);

    assert!(tt[0].hypot(tt[1]) < 0.25);

    Ok(())
}
