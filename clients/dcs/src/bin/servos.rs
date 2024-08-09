use gmt_dos_actors::{actorscript, system::Sys};
use gmt_dos_clients::Timer;
use gmt_dos_clients_dcs::{
    mount_trajectory::{
        ImMountTrajectory, MountTrajectory, OcsMountTrajectory, RelativeMountTrajectory,
    },
    Dcs, Pull, Push,
};
use gmt_dos_clients_io::{
    cfd_wind_loads::{CFDM1WindLoads, CFDM2WindLoads, CFDMountWindLoads},
    mount::{AverageMountEncoders, MountSetPoint},
    M12RigidBodyMotions,
};
use gmt_dos_clients_servos::{GmtFem, GmtMount, GmtServoMechanisms};
use gmt_dos_clients_transceiver::{Monitor, Transceiver};
use gmt_dos_clients_windloads::system::{Mount, SigmoidCfdLoads, M1, M2};
use interface::{filing::Filing, Tick};
use nanomsg::Socket;

const PULL: &str = "tcp://127.0.0.1:4242";
const PUSH: &str = "tcp://127.0.0.1:4243";
const ACTUATOR_RATE: usize = 80;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_level(false)
        .format_timestamp_millis()
        .init();

    let gmt_servos = Sys::<GmtServoMechanisms<ACTUATOR_RATE, 1>>::from_path(
        "preloaded_servos_zen30az000_OS7.bin",
    )?;

    let windloads = Sys::<SigmoidCfdLoads>::from_path("preloaded_windloads_zen30az000_OS7.bin")?;

    let dcs_pull = Dcs::<Pull, Socket, MountTrajectory>::new(PULL)?;
    let dcs_push = Dcs::<Push, Socket, MountTrajectory>::new(PUSH)?;

    let rmt = RelativeMountTrajectory::default();

    let metronome: Timer = Timer::new(300);

    let mut tx_monitor = Monitor::new();
    let address = "127.0.0.1";
    let tx = Transceiver::<M12RigidBodyMotions>::transmitter(address)?.run(&mut tx_monitor);

    actorscript!(
        #[labels(
            dcs_pull = "From OCS", dcs_push = "To OCS",
            scope_mountsetpoint = "Scope", scope_averagemountencoders = "Scope",
            rmt = "Relative\nTrajectory",
            tx = "TX"
        )]

        1: {windloads::M1}[CFDM1WindLoads] -> {gmt_servos::GmtFem}
        1: {windloads::M2}[CFDM2WindLoads] -> {gmt_servos::GmtFem}
        1: {windloads::Mount}[CFDMountWindLoads] -> {gmt_servos::GmtFem}

        50: metronome[Tick] -> dcs_pull[OcsMountTrajectory]${3} -> rmt[MountSetPoint]~
        50: rmt[ImMountTrajectory]${3} -> dcs_push
        1: rmt[MountSetPoint] -> {gmt_servos::GmtMount}
        1: {gmt_servos::GmtFem}[AverageMountEncoders]! -> rmt
        50: {gmt_servos::GmtFem}[AverageMountEncoders]!~
        8: {gmt_servos::GmtFem}[M12RigidBodyMotions].. -> tx
    );

    tx_monitor.await?;

    Ok(())
}
