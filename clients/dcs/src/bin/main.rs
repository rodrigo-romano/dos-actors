use gmt_dos_actors::actorscript;
use gmt_dos_clients_dcs::{
    mount_trajectory::{
        MountTrajectory, OcsMountTrajectory, RelativeMountAxes, RelativeMountTrajectory,
    },
    Dcs, Pull, Push,
};
use interface::units::Mas;
use nanomsg::Socket;

const PULL: &str = "tcp://127.0.0.1:4242";
const PUSH: &str = "tcp://127.0.0.1:4243";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let dcs_pull = Dcs::<Pull, Socket, MountTrajectory>::new(PULL)?;
    let dcs_push = Dcs::<Push, Socket, MountTrajectory>::new(PUSH)?;

    let rmt = RelativeMountTrajectory::default();

    actorscript!(
        1:  dcs_pull[OcsMountTrajectory].. -> dcs_push
        1:  dcs_pull[OcsMountTrajectory].. -> rmt[Mas<RelativeMountAxes>]~
    );

    Ok(())
}
