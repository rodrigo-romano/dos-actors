use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{
    interface::{Data, Read, UniqueIdentifier, Write},
    Tick, Timer,
};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{fem_io::actors_outputs::*, DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::{
    gmt_m1::M1RigidBodyMotions,
    gmt_m2::M2RigidBodyMotions,
    mount::{MountEncoders, MountTorques},
};
use gmt_dos_clients_transceiver::{Crypto, Monitor, On, Receiver, Transceiver, Transmitter};
use gmt_fem::FEM;
use gmt_lom::{OpticalMetrics, LOM};
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

impl Read<Toc> for Transceiver<MountTorques, Receiver, On> {
    fn read(&mut self, data: Data<Toc>) {
        self.rx = None;
    }
}

// Move the mount 1arcsec along the elevation axis of the telescope
// DATA:
//  * FEM 2nd order model: FEM_REPO
//  * linear optical sensitivity matrices: LOM

// cargo test --release --package gmt_dos-clients_mount --test setpoint_mount --features mount-fdr -- setpoint_mount --exact --nocapture
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /*     tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap(); */
    env_logger::init();

    let sim_sampling_frequency = 1000;
    let sim_duration = 20_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    // FEM MODEL
    let state_space = {
        let fem = FEM::from_env()?;
        println!("{fem}");
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            //.max_eigen_frequency(75f64)
            .including_mount()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .use_static_gain_compensation()
            .build()?
    };
    println!("{state_space}");

    let mut monitor = Monitor::new();
    let fem_crypo = Crypto::builder()
        .certificate("fem_cert.der")
        .key("fem_key.der")
        .build();
    let mut encoders: Terminator<_> =
        Transceiver::<MountEncoders>::transmitter_builder("127.0.0.1:5001")
            .crypto(fem_crypo)
            .build()?
            .run(&mut monitor)
            .into();
    let mount_crypo = Crypto::builder()
        .certificate("mount_cert.der")
        .key("mount_key.der")
        .build();
    let mut torques: Initiator<_> =
        Transceiver::<MountTorques>::receiver_builder("127.0.0.1:5002", "127.0.0.1:0")
            .crypto(mount_crypo)
            .build()?
            .run(&mut monitor)
            .into();

    // FEM
    let mut fem: Actor<_> = state_space.into();
    // Logger
    let logging = Arrow::builder(n_step).build().into_arcx();
    let mut sink = Terminator::<_>::new(logging.clone());

    torques
        .add_output()
        .build::<MountTorques>()
        .into_input(&mut fem)?;
    fem.add_output()
        .bootstrap()
        .build::<MountEncoders>()
        .into_input(&mut encoders)?;
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

    model!(torques, encoders, fem, sink)
        .name("fem-server")
        .check()?
        .flowchart()
        .run()
        .wait()
        .await?;

    monitor.await?;

    // Linear optical sensitivities to derive segment tip and tilt
    let lom = LOM::builder()
        .rigid_body_motions_record(
            (*logging.lock().await).record()?,
            Some("M1RigidBodyMotions"),
            Some("M2RigidBodyMotions"),
        )?
        .build()?;
    let segment_tiptilt = lom.segment_tiptilt();
    let stt = segment_tiptilt.items().last().unwrap();

    println!("Segment TT: {:.3?}mas", stt.to_mas());
    //assert!(tt[0].hypot(tt[1]) < 0.25);

    Ok(())
}
