use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Signal, Signals};
use gmt_dos_clients_io::mount::{MountEncoders, MountSetPoint, MountTorques};
use gmt_dos_clients_mount::Mount;
use gmt_dos_clients_transceiver::{Monitor, Transceiver};
use skyangle::Conversion;

// Move the mount 1arcsec along the elevation axis of the telescope
// DATA:
//  * FEM 2nd order model: FEM_REPO
//  * linear optical sensitivity matrices: LOM

// cargo test --release --package gmt_dos-clients_mount --test setpoint_mount --features mount-fdr -- setpoint_mount --exact --nocapture
#[tokio::test]
async fn setpoint_mount_dist() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 1000;
    let sim_duration = 20_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mut monitor = Monitor::new();
    let mut encoders: Initiator<_> =
        Transceiver::<MountEncoders>::receiver("127.0.0.1:5001", "127.0.0.1:0")?
            .run(&mut monitor)
            .into();
    let mut torques: Terminator<_> = Transceiver::<MountTorques>::transmitter("127.0.0.1:5002")?
        .run(&mut monitor)
        .into();

    // SET POINT
    let mut setpoint: Initiator<_> = Signals::new(3, n_step)
        .channel(1, Signal::Constant(1f64.from_arcsec()))
        .into();
    // MOUNT CONTROL
    let mut mount: Actor<_> = Mount::new().into();

    setpoint
        .add_output()
        .build::<MountSetPoint>()
        .into_input(&mut mount)?;
    mount
        .add_output()
        .build::<MountTorques>()
        .into_input(&mut torques)?;
    encoders
        .add_output()
        .build::<MountEncoders>()
        .into_input(&mut mount)?;

    model!(setpoint, mount, torques, encoders)
        .check()?
        .flowchart()
        .run()
        .wait()
        .await?;

    monitor.await?;

    Ok(())
}
