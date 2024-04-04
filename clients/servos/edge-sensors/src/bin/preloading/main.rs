/*!
# M2 ASMS Edge Sensors Feed-Forward and Shell Off-load

Model of the GMT servo-mechanisms with M2 edge sensors feed-forward to the the
voice coil actuators and off-load from the voice coil actuators to the reference bodies.

*/
use anyhow::Result;
use gmt_dos_actors::{actorscript, system::Sys};
use gmt_dos_clients::Timer;
use gmt_dos_clients_io::cfd_wind_loads::{CFDM1WindLoads, CFDM2WindLoads, CFDMountWindLoads};
use gmt_dos_clients_servos::{
    asms_servo, AsmsServo, EdgeSensors, GmtFem, GmtServoMechanisms, WindLoads,
};
use gmt_dos_clients_windloads::{
    system::{Mount, SigmoidCfdLoads, M1, M2},
    CfdLoads,
};
use gmt_fem::FEM;
use interface::{filing::Filing, Tick};
use std::{env, path::Path};

const ACTUATOR_RATE: usize = 80; // 100Hz
const PRELOADING_N_SAMPLE: usize = 8000 * 3;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp(None).init();

    let data_repo = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("bin")
        .join("m2");
    env::set_var("DATA_REPO", &data_repo);
    let fem_var = env::var("FEM_REPO").expect("`FEM_REPO` is not set");
    let _fem_path = Path::new(&fem_var);

    let sim_sampling_frequency = 8000;

    // GMT Servo-Mechanisms
    let (cfd_loads, gmt_servos) = {
        let mut fem = Option::<FEM>::None;
        // The CFD wind loads must be called next afer the FEM as it is modifying the FEM CFDMountWindLoads inputs
        let cfd_loads = Sys::<SigmoidCfdLoads>::from_data_repo_or("preloaded_windloads.bin", {
            CfdLoads::foh(data_repo.to_str().unwrap(), sim_sampling_frequency)
                .duration(10f64)
                .mount(fem.get_or_insert_with(|| FEM::from_env().unwrap()), 0, None)
                .m1_segments()
                .m2_segments()
        })?;

        let gmt_servos = Sys::<GmtServoMechanisms<ACTUATOR_RATE, 1>>::from_data_repo_or_else(
            "preloaded_servos.bin",
            || {
                GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(
                    sim_sampling_frequency as f64,
                    fem.unwrap(),
                )
                .wind_loads(WindLoads::new())
                .asms_servo(AsmsServo::new().reference_body(asms_servo::ReferenceBody::new()))
                .edge_sensors(EdgeSensors::both())
            },
        )?;

        (cfd_loads, gmt_servos)
    };

    let metronome: Timer = Timer::new(PRELOADING_N_SAMPLE);

    actorscript! {
    1: metronome[Tick] -> {gmt_servos::GmtFem}

    1: {cfd_loads::M1}[CFDM1WindLoads] -> {gmt_servos::GmtFem}
    1: {cfd_loads::M2}[CFDM2WindLoads] -> {gmt_servos::GmtFem}
    1: {cfd_loads::Mount}[CFDMountWindLoads] -> {gmt_servos::GmtFem}
    }
    gmt_servos.to_data_repo("preloaded_servos.bin")?;
    cfd_loads.to_data_repo("preloaded_windloads.bin")?;

    Ok(())
}
