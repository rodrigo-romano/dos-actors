use gmt_dos_actors::actorscript;
use gmt_dos_clients::{Signal, Signals};
use gmt_dos_clients_fem::{fem_io::actors_outputs::*, DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::{
    // gmt_m1::M1RigidBodyMotions,
    // gmt_m2::M2RigidBodyMotions,
    mount::{MountEncoders, MountSetPoint, MountTorques},
};
use gmt_dos_clients_mount::Mount;
use gmt_fem::FEM;
// use gmt_lom::{OpticalMetrics, LOM};
use skyangle::Conversion;

/*
DATA:
 * FEM 2nd order model: FEM_REPO
 * linear optical sensitivity matrices: LOM

MOUNT_MODEL=... cargo test --release --package gmt_dos-clients_mount --test main -- --nocapture
*/

async fn set_mount(
    sim_sampling_frequency: usize,
    setpoint: Signals,
) -> anyhow::Result<((f64, f64), (f64, f64), (f64, f64))> {
    // FEM MODEL
    let state_space = {
        let fem = FEM::from_env()?;
        println!("{fem}");
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .including_mount()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .use_static_gain_compensation()
            .build()?
    };
    // println!("{state_space}");

    // FEM
    let fem = state_space;
    // MOUNT CONTROL
    let mount = Mount::new();

    actorscript! {
        #[labels(fem = "FEM", mount = "Mount\nControl")]
        1: setpoint[MountSetPoint] -> mount[MountTorques] -> fem[MountEncoders]!${14} -> mount
        // 1: fem[M1RigidBodyMotions]$
        // 1: fem[M2RigidBodyMotions]$
    }

    let mut logs = logging_1.lock().await;
    let encs: Vec<f64> = logs.iter("MountEncoders")?.last().unwrap();

    let stats = |x: &[f64]| {
        let (mut mean, mut var) = x.iter().fold((0f64, 0f64), |(a, b), x| {
            let a = a + x;
            let b = b + x * x;
            (a, b)
        });
        let n = x.len() as f64;
        mean /= n;
        var /= n;
        var -= mean * mean;
        let std = var.sqrt();
        (mean.to_mas(), std.to_mas())
    };

    let (azimuth, elevation, gir) = (stats(&encs[..4]), stats(&encs[4..10]), stats(&encs[10..]));
    println!("Azimuth: {azimuth:.3?}mas");
    println!("Elevation: {elevation:.3?}mas");
    println!("GIR: {gir:.3?}mas");

    /*     // Linear optical sensitivities to derive segment tip and tilt
    let lom = LOM::builder()
        .rigid_body_motions_record(
            logs.record()?,
            Some("M1RigidBodyMotions"),
            Some("M2RigidBodyMotions"),
        )?
        .build()?;
    let segment_tiptilt = lom.segment_tiptilt();
    let stt = segment_tiptilt.items().last().unwrap();
    let tiptilt = lom.tiptilt();
    let tt = tiptilt.items().last().unwrap();

    println!("Segment TT: {:.3?}mas", stt.to_mas());
    println!("TT: {:.3?}mas", tt.to_mas());
    // assert!(tt[0].hypot(tt[1]).to_mas() - 1000. < 1.); */

    Ok((azimuth, elevation, gir))
}

/// Moves the mount 1arcsec along the elevation axis of the telescope
///
/// The test is succesfull if the last position is less than 1arcsec to the target
#[tokio::test]
async fn elevation() -> anyhow::Result<()> {
    let sim_sampling_frequency = gmt_dos_clients_mount::sampling_frequency();
    let sim_duration = 4_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;
    // SET POINT
    let setpoint = Signals::new(3, n_step).channel(1, Signal::Constant(1f64.from_arcsec()));
    let (_, enc, _) = set_mount(sim_sampling_frequency, setpoint).await?;
    assert!((enc.0 - 1000.).abs() < 1.);
    Ok(())
}

/// Moves the mount 1arcsec along the azimuth axis of the telescope
///
/// The test is succesfull if the last position is less than 1arcsec to the target
#[tokio::test]
async fn azimuth() -> anyhow::Result<()> {
    let sim_sampling_frequency = gmt_dos_clients_mount::sampling_frequency();
    let sim_duration = 4_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;
    // SET POINT
    let setpoint = Signals::new(3, n_step).channel(0, Signal::Constant(1f64.from_arcsec()));
    let (enc, _, _) = set_mount(sim_sampling_frequency, setpoint).await?;
    assert!((enc.0 - 1000.).abs() < 1.);
    Ok(())
}

/// Moves the mount 1arcsec along the GIR axis of the telescope
///
/// The test is succesfull if the last position is less than 1arcsec to the target
#[tokio::test]
async fn gir() -> anyhow::Result<()> {
    let sim_sampling_frequency = gmt_dos_clients_mount::sampling_frequency();
    let sim_duration = 4_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;
    // SET POINT
    let setpoint = Signals::new(3, n_step).channel(2, Signal::Constant(1f64.from_arcsec()));
    let (_, _, enc) = set_mount(sim_sampling_frequency, setpoint).await?;
    assert!((enc.0 - 1000.).abs() < 1.);
    Ok(())
}

/// Zero command test
///
/// The test is succesfull if the last position of the 3 axis is less than 1/4arcsec to zero
#[tokio::test]
async fn zero_mount() -> anyhow::Result<()> {
    let sim_sampling_frequency = gmt_dos_clients_mount::sampling_frequency();
    let sim_duration = 4_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;
    // SET POINT
    let setpoint = Signals::new(3, n_step);
    let (az, el, gir) = set_mount(sim_sampling_frequency, setpoint).await?;
    assert!(az.0 < 0.25);
    assert!(el.0 < 0.25);
    assert!(gir.0 < 0.25);

    Ok(())
}
