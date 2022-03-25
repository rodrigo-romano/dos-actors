use dos_actors::clients::mount::{Mount, MountEncoders, MountSetPoint, MountTorques};
use dos_actors::{clients::arrow_client::Arrow, prelude::*};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use lom::{OpticalMetrics, LOM};
use skyangle::Conversion;
use std::time::Instant;

#[tokio::test]
async fn setpoint_mount() -> anyhow::Result<()> {
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
        .add_output()
        .build::<D, MountSetPoint>()
        .into_input(&mut mount);
    mount
        .add_output()
        .build::<D, MountTorques>()
        .into_input(&mut fem);
    fem.add_output()
        .bootstrap()
        .build::<D, MountEncoders>()
        .into_input(&mut mount);
    fem.add_output()
        .unbounded()
        .build::<D, OSSM1Lcl>()
        .into_input(&mut sink);
    fem.add_output()
        .unbounded()
        .build::<D, MCM2Lcl6D>()
        .into_input(&mut sink);

    let now = Instant::now();
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
    println!("Elapsed time {}ms", now.elapsed().as_millis());

    let lom = LOM::builder()
        .rigid_body_motions_record((*logging.lock().await).record()?)?
        .build()?;
    let segment_tiptilt = lom.segment_tiptilt();
    let stt = segment_tiptilt.items().last().unwrap();

    println!("Segment TT: {:.3?}mas", stt.to_mas());
    //assert!(tt[0].hypot(tt[1]) < 0.25);

    Ok(())
}
