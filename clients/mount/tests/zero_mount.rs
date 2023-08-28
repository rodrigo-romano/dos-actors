//! Mount controller null test
//!
//! Run the mount controller with the mount torques and encoders of the FEM model
//! and with the mount control set points set to 0
//! The FEM model repository is read from the `FEM_REPO` environment variable
//! The LOM sensitivity matrices are located in the directory given by the `LOM` environment variable

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::Signals;
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{
    fem_io::{actors_inputs::*, actors_outputs::*},
    DiscreteModalSolver, ExponentialMatrix,
};
use gmt_dos_clients_io::{
    gmt_m1::M1RigidBodyMotions,
    gmt_m2::M2RigidBodyMotions,
    mount::{MountEncoders, MountSetPoint, MountTorques},
};
use gmt_dos_clients_mount::Mount;
use gmt_fem::FEM;
use gmt_lom::{Stats, LOM};

// cargo test --release --package gmt_dos-clients_mount --test zero_mount --all-features -- zero_mount_at --exact --nocapture
#[tokio::test]
async fn zero_mount_at() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000;
    let sim_duration = 4_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    let state_space = {
        let fem = FEM::from_env()?;
        println!("{fem}");
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
            .use_static_gain_compensation()
            .build()?
    };

    let mut source: Initiator<_> = Signals::new(3, n_step).into();
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT
    let mut mount: Actor<_> = Mount::new().into();

    let logging = Arrow::builder(n_step).build().into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    source
        .add_output()
        .build::<MountSetPoint>()
        .into_input(&mut mount)?;
    mount
        .add_output()
        .build::<MountTorques>()
        .into_input(&mut fem)?;
    fem.add_output()
        .bootstrap()
        .multiplex(2)
        .build::<MountEncoders>()
        .into_input(&mut mount)
        .logn(&mut sink, 14)
        .await?;
    fem.add_output()
        .unbounded()
        .build::<M1RigidBodyMotions>()
        .log(&mut sink)
        .await?;
    fem.add_output()
        .unbounded()
        .build::<M2RigidBodyMotions>()
        .log(&mut sink)
        .await?;

    model!(source, mount, fem, sink)
        .name("mount")
        .flowchart()
        .check()?
        .run()
        .wait()
        .await?;

    let lom = LOM::builder()
        .rigid_body_motions_record(
            (*logging.lock().await).record()?,
            Some("M1RigidBodyMotions"),
            Some("M2RigidBodyMotions"),
        )?
        .build()?;
    let tiptilt = lom.tiptilt_mas();
    let n_sample = 1000;
    let tt = tiptilt.std(Some(n_sample));
    println!("TT STD.: {:.3?}mas", tt);

    assert!(tt[0].hypot(tt[1]) < 0.25);

    Ok(())
}
