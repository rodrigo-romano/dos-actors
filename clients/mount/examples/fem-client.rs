use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{
    interface::{Data, Read, UniqueIdentifier, Write},
    Signal, Signals, Timer,
};
use gmt_dos_clients_io::mount::{MountEncoders, MountSetPoint, MountTorques};
use gmt_dos_clients_mount::Mount;
use gmt_dos_clients_transceiver::{Crypto, Monitor, On, Receiver, Transceiver};
use skyangle::Conversion;

pub enum Toc {}
impl UniqueIdentifier for Toc {
    type DataType = ();
}

impl Write<Toc> for Timer {
    fn write(&mut self) -> Option<Data<Toc>> {
        if self.tick > 0 {
            Some(Data::new(()))
        } else {
            None
        }
    }
}

impl Read<Toc> for Transceiver<MountEncoders, Receiver, On> {
    fn read(&mut self, data: Data<Toc>) {
        self.rx.take().map(|rx| {
            let _ = drop(rx);
        });
    }
}

// Move the mount 1arcsec along the elevation axis of the telescope
// DATA:
//  * FEM 2nd order model: FEM_REPO
//  * linear optical sensitivity matrices: LOM

// cargo test --release --package gmt_dos-clients_mount --test setpoint_mount --features mount-fdr -- setpoint_mount --exact --nocapture
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing::subscriber::set_global_default(
    //     tracing_subscriber::FmtSubscriber::builder()
    //         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    //         .finish(),
    // )
    // .unwrap();
    env_logger::init();

    let sim_sampling_frequency = 1000;
    let sim_duration = 20_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let fem_crypo = Crypto::builder()
        .certificate("fem_cert.der")
        .key("fem_key.der")
        .build();
    let mount_crypo = Crypto::builder()
        .certificate("mount_cert.der")
        .key("mount_key.der")
        .build();

    let mut timer: Initiator<_> = Timer::new(n_step).into();

    let mut monitor: Monitor = Monitor::new();
    let mut encoders: Actor<_> =
        Transceiver::<MountEncoders>::receiver_builder("127.0.0.1:5001", "127.0.0.1:0")
            .crypto(fem_crypo)
            .build()?
            .run(&mut monitor)
            .into();
    let mut torques: Terminator<_> =
        Transceiver::<MountTorques>::transmitter_builder("127.0.0.1:5002")
            .crypto(mount_crypo)
            .build()?
            .run(&mut monitor)
            .into();

    // SET POINT
    let mut setpoint: Initiator<_> = Signals::new(3, n_step)
        .channel(1, Signal::Constant(1f64.from_arcsec()))
        .into();
    // MOUNT CONTROL
    let mut mount: Actor<_> = Mount::new().into();

    timer
        .add_output()
        .build::<Toc>()
        .into_input(&mut encoders)?;

    setpoint
        .add_output()
        .unbounded()
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

    model!(setpoint, mount, torques, encoders, timer)
        .name("mount-client")
        .check()?
        .flowchart()
        .run()
        .wait()
        .await?;

    monitor.await?;

    Ok(())
}
