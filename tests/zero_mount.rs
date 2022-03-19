use std::time::Instant;

use dos_actors::clients::mount::{Mount, MountEncoders, MountSetPoint, MountTorques};
use dos_actors::{clients::arrow_client::Arrow, prelude::*};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use futures::future::join_all;
use gmt_lom::{Stats, Table, LOM};

#[tokio::test]
async fn zero_mount() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000;
    let sim_duration = 4_usize;
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

    let mut source: Initiator<_> = Signals::new(vec![3], n_step).into();
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT
    let mut mount: Actor<_> = Mount::new().into();

    let logging = Arrow::builder(n_step)
        .entry::<f64, OSSM1Lcl>(42)
        .entry::<f64, MCM2Lcl6D>(42)
        .no_save()
        .build()
        .into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    type D = Vec<f64>;
    source
        .add_single_output()
        .build::<D, MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_single_output()
        .build::<D, MountTorques>()
        .into_input(&mut fem);
    fem.add_single_output()
        .bootstrap()
        .build::<D, MountEncoders>()
        .into_input(&mut mount);
    fem.add_single_output()
        .unbounded()
        .build::<D, OSSM1Lcl>()
        .into_input(&mut sink);
    fem.add_single_output()
        .unbounded()
        .build::<D, MCM2Lcl6D>()
        .into_input(&mut sink);

    let now = Instant::now();
    let tasks = vec![source.spawn(), mount.spawn(), fem.spawn(), sink.spawn()];
    join_all(tasks).await;
    println!("Elapsed time {}ms", now.elapsed().as_millis());

    let table: Table = (*logging.lock().await).record()?.into();
    let lom = LOM::builder().table_rigid_body_motions(&table)?.build()?;
    let tiptilt = lom.tiptilt();
    let n_sample = 1000;
    let tt = tiptilt.std(Some(n_sample));
    println!("TT STD.: {:.3?}mas", tt);

    assert!(tt[0].hypot(tt[1]) < 0.25);

    Ok(())
}
