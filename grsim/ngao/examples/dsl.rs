use std::{env, path::Path, time::Instant};

use crseo::{
    wavefrontsensor::{PistonSensor, SegmentCalibration},
    Builder, FromBuilder, Gmt, SegmentWiseSensorBuilder, WavefrontSensorBuilder,
};
use gmt_dos_actors::prelude::*;
use gmt_dos_actors_dsl::actorscript;
use gmt_dos_clients::{Integrator, Tick, Timer};
use gmt_dos_clients_crseo::{M2modes, SegmentPiston};
use gmt_dos_clients_scope::server;
use ngao::{GuideStar, LittleOpticalModel, ResidualPistonMode, WavefrontSensor};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data_repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    dbg!(&data_repo);
    env::set_var("DATA_REPO", &data_repo);
    env::set_var(
        "GMT_MODES_PATH",
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data"),
    );

    let sampling_frequency = 1_000usize; // Hz
    let sim_duration = 1usize;
    let n_sample = sim_duration * sampling_frequency;

    let m2_modes: &str = "M2_OrthoNormGS36_KarhunenLoeveModes";

    let pupil_sampling = 92 * 4;
    let piston_builder = PistonSensor::builder().pupil_sampling(pupil_sampling);
    let src_builder = piston_builder.guide_stars(None);
    let now = Instant::now();
    let mut piston_mat = piston_builder.calibrate(
        SegmentCalibration::modes(m2_modes, 0..1, "M2"),
        src_builder.clone(),
    );
    println!(
        "M2 {}modes/segment calibrated in {}s",
        1,
        now.elapsed().as_secs()
    );
    piston_mat.pseudo_inverse(None).unwrap();

    let m2_modes = "M2_OrthoNormGS36_KarhunenLoeveModes";
    let gom = LittleOpticalModel::builder()
        .gmt(Gmt::builder().m2(m2_modes, 1))
        .source(src_builder)
        .atmosphere(
            crseo::Atmosphere::builder().ray_tracing(
                25.5,
                pupil_sampling as i32,
                0f32,
                sim_duration as f32,
                Some(
                    data_repo
                        .join("ngao_atmophere.bin")
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
                None,
            ),
        )
        .sampling_frequency(sampling_frequency as f64)
        .build()?;

    let mut timer: Timer = Timer::new(n_sample);
    let mut piston_sensor = WavefrontSensor::new(piston_builder.build()?, piston_mat);
    let mut ctrl = Integrator::new(7).gain(0.5);

    let mut monitor = server::Monitor::new();
    let scope = server::Scope::<SegmentPiston>::builder("172.31.26.127:5001", &mut monitor)
        .sampling_period((sampling_frequency as f64).recip())
        .scale(1e9)
        .build()?;

    actorscript! {
        timer<Tick>
        -> &gom<GuideStar>
        -> piston_sensor<ResidualPistonMode>
        -> ctrl[bootstrap]<M2modes>
        -> &gom[unbounded]<SegmentPiston>
        -> scope
    }
    model
        .name("piston_sensor")
        .flowchart()
        .check()?
        .run()
        .await?;

    monitor.await?;

    Ok(())
}
