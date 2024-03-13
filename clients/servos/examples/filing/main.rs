use std::{env, path::Path};

use gmt_dos_actors::system::Sys;
use gmt_dos_clients_servos::{AsmsServo, GmtServoMechanisms};
use gmt_fem::FEM;
use interface::filing::Filing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    env::set_var(
        "DATA_REPO",
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("filing"),
    );

    let sim_sampling_frequency = 8000;

    let now = std::time::Instant::now();
    let gmt_servos =
        Sys::<GmtServoMechanisms<80, 1>>::from_data_repo_or_else("servos.bin", || {
            GmtServoMechanisms::<80, 1>::new(
                sim_sampling_frequency as f64,
                FEM::from_env().unwrap(),
            )
            .asms_servo(AsmsServo::new().facesheet(Default::default()))
        })?;
    println!("loading elapsed time: {:.3?}", now.elapsed());

    Ok(())
}
