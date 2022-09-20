use gmt_dos_actors::clients::mount::{Mount, MountEncoders, MountSetPoint, MountTorques};
use gmt_dos_actors::{clients::arrow_client::Arrow, prelude::*};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use lom::{OpticalMetrics, LOM};
use skyangle::Conversion;

#[tokio::test]
async fn setpoint_mount() -> anyhow::Result<()> {
    let sim_sampling_frequency = 8000;
    let sim_duration = 1_usize;
    let n_step = sim_sampling_frequency * sim_duration;

    let state_space = {
        let fem = FEM::from_env()?.static_from_env()?;
        println!("{fem}");
        let n_io = (fem.n_inputs(), fem.n_outputs());
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            //.max_eigen_frequency(75f64)
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
        .output_signal(1, Signal::Constant(1f64.from_arcsec()))
        .into();
    // FEM
    let mut fem: Actor<_> = state_space.into();
    // MOUNT
    let mut mount: Actor<_> = Mount::new().into();
    let logging = Arrow::builder(n_step).build().into_arcx();
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
        .multiplex(2)
        .build::<MountEncoders>()
        .into_input(&mut mount)
        .logn(&mut sink, 14)
        .await;
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
        .rigid_body_motions_record(
            (*logging.lock().await).record()?,
            Some("OSSM1Lcl"),
            Some("MCM2Lcl6D"),
        )?
        .build()?;
    let segment_tiptilt = lom.segment_tiptilt();
    let stt = segment_tiptilt.items().last().unwrap();

    println!("Segment TT: {:.3?}mas", stt.to_mas());
    //assert!(tt[0].hypot(tt[1]) < 0.25);

    Ok(())
}
