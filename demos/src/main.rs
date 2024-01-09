use gmt_dos_actors::{actorscript};
use gmt_dos_clients::{Signal, Signals};
use gmt_dos_clients_fem::{fem_io::actors_outputs::*, DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::{
    gmt_m1::M1RigidBodyMotions,
    gmt_m2::M2RigidBodyMotions,
    mount::{MountEncoders, MountSetPoint, MountTorques},
    optics::TipTilt
};
use gmt_dos_clients_mount::Mount;
use gmt_fem::FEM;
use skyangle::Conversion;
use gmt_dos_clients_lom::LinearOpticalModel;
use interface::units::Arcsec;

// Move the mount 1arcsec along the elevation axis of the telescope
// DATA:
//  * FEM 2nd order model: FEM_REPO
//  * linear optical sensitivity matrices: LOM

// cargo test --release --package gmt_dos-clients_mount --test setpoint_mount --features mount-pdr -- setpoint_mount --exact --nocapture

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let sim_sampling_frequency = 8000;
    let sim_duration = 4_usize; // second
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

    // SET POINT
    let setpoint = Signals::new(3, n_step).channel(1, Signal::Constant(1f64.from_arcsec()));
    // FEM
    let fem = state_space;
    // MOUNT CONTROL
    let mount = Mount::new();
    // LOM
    let lom = LinearOpticalModel::new()?;

    actorscript! {
        #[labels(fem = "GMT FEM", mount = "Mount\nControl", lom="Linear Optical\nModel")]
        1: setpoint[MountSetPoint] -> mount[MountTorques] -> fem[MountEncoders]! -> mount
        1: fem[M1RigidBodyMotions] -> lom
        1: fem[M2RigidBodyMotions] -> lom[Arcsec<TipTilt>]~
    }

    Ok(())
}
