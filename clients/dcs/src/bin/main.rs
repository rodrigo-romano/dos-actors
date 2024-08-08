use gmt_dos_actors::actorscript;
use gmt_dos_clients::Timer;
use gmt_dos_clients_dcs::{
    mount_trajectory::{
        ImMountTrajectory, MountTrajectory, OcsMountTrajectory, RelativeMountTrajectory,
    },
    Dcs, Pull, Push,
};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::{
    gmt_fem::outputs::{MCM2Lcl6D, OSSM1Lcl},
    mount::{AverageMountEncoders, MountEncoders, MountSetPoint, MountTorques},
};
use gmt_dos_clients_mount::Mount;
use interface::Tick;
use nanomsg::Socket;

const PULL: &str = "tcp://127.0.0.1:4242";
const PUSH: &str = "tcp://127.0.0.1:4243";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 1000_usize;

    let fem = DiscreteModalSolver::<ExponentialMatrix>::from_env()?
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        .including_mount()
        .outs::<OSSM1Lcl>()
        .outs::<MCM2Lcl6D>()
        .use_static_gain_compensation()
        .build()?;

    let mount = Mount::new();

    let dcs_pull = Dcs::<Pull, Socket, MountTrajectory>::new(PULL)?;
    let dcs_push = Dcs::<Push, Socket, MountTrajectory>::new(PUSH)?;

    let rmt = RelativeMountTrajectory::default();

    let metronome: Timer = Timer::new(100);

    actorscript!(
        #[labels(fem = "60deg EL\n0deg AZ")]
        #[images(fem = "gmt-pretty4.png")]
        50: metronome[Tick] -> dcs_pull[OcsMountTrajectory]${3} -> rmt[MountSetPoint]
        50: rmt[ImMountTrajectory]${3} -> dcs_push
        1: rmt[MountSetPoint] -> mount[MountTorques] -> fem[MountEncoders]! -> mount
        1: fem[AverageMountEncoders]! -> rmt
    );

    Ok(())
}
