use gmt_dos_actors::actorscript;
use gmt_dos_clients::{Gain, Timer};
use gmt_dos_clients_dcs::{
    mount_trajectory::{MountTrajectory, OcsMountTrajectory, RelativeMountTrajectory},
    Dcs, Pull, Push,
};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::{
    gmt_fem::outputs::{MCM2Lcl6D, OSSM1Lcl},
    mount::{AverageMountEncoders, MountEncoders, MountSetPoint, MountTorques},
};
use gmt_dos_clients_mount::Mount;
use interface::{
    units::{Arcsec, Deg},
    Tick,
};
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

    let to_deg1 = Gain::new(vec![180.0 / std::f64::consts::PI; 3]);
    let to_deg2 = Gain::new(vec![180.0 / std::f64::consts::PI; 3]);

    let metronome: Timer = Timer::new(1000);

    actorscript!(
        50: metronome[Tick] -> dcs_pull[OcsMountTrajectory].. -> dcs_push
        50: dcs_pull[OcsMountTrajectory].. -> rmt[MountSetPoint]~
        1: rmt[MountSetPoint] -> mount[MountTorques] -> fem[MountEncoders]! -> mount
        50: fem[AverageMountEncoders]~
        //1: dcs_pull[OcsMountTrajectory].. -> to_deg1[OcsMountTrajectory]~
    );

    Ok(())
}
