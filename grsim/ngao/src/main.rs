use std::time::Instant;

use crseo::{
    wavefrontsensor::{PhaseSensor, PhaseSensorBuilder, SegmentCalibration},
    Builder, FromBuilder, Gmt, SegmentWiseSensorBuilder, WavefrontSensorBuilder,
};
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Integrator, Logging, Tick, Timer};
use gmt_dos_clients_crseo::{
    M2modes, OpticalModel, OpticalModelOptions, SegmentWfeRms, SensorData,
};
use ngao::{GuideStar, LittleOpticalModel, WavefrontSensor};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let n_lenslet = 92;
    let n_mode = 250;

    let builder = PhaseSensor::builder().lenslet(n_lenslet, 16);
    let src_builder = builder.guide_stars(None);

    let now = Instant::now();
    let mut slopes_mat = builder.clone().calibrate(
        SegmentCalibration::modes("Karhunen-Loeve", 0..n_mode, "M2"),
        src_builder.clone(),
    );
    println!(
        "M2 {}modes/segment calibrated in {}s",
        n_mode,
        now.elapsed().as_secs()
    );
    slopes_mat.pseudo_inverse().unwrap();

    let atm_sampling_frequency = 500usize; // Hz

    let gom = LittleOpticalModel::builder()
        .gmt(Gmt::builder().m2("Karhunen-Loeve", n_mode))
        .source(src_builder)
        .atmosphere(crseo::Atmosphere::builder())
        .sampling_frequency(atm_sampling_frequency as f64)
        .build()?
        .into_arcx();

    let n_sample = 10;

    let mut gom_act: Actor<_> = Actor::new(gom.clone());

    let mut sensor: Actor<_> = WavefrontSensor::new(builder.build()?, slopes_mat).into();

    let mut timer: Initiator<_> = Timer::new(n_sample).into();
    let mut feedback: Actor<_> = Integrator::new(n_mode * 7)
        // .chunks(n_mode)
        // .skip(1)
        .gain(0.5)
        .into();
    let logging = Logging::new(1).into_arcx();
    let mut logger: Terminator<_> = Actor::new(logging.clone());

    timer
        .add_output()
        .build::<Tick>()
        .into_input(&mut gom_act)?;
    gom_act
        .add_output()
        .build::<GuideStar>()
        .into_input(&mut sensor)?;
    sensor
        .add_output()
        .build::<M2modes>()
        .into_input(&mut feedback)?;
    feedback
        .add_output()
        .bootstrap()
        .build::<M2modes>()
        .into_input(&mut gom_act)?;
    gom_act
        .add_output()
        .build::<SegmentWfeRms>()
        .into_input(&mut logger)?;

    model!(timer, feedback, gom_act, sensor, logger)
        .name("NGAO")
        .flowchart()
        .check()?
        .run()
        .await?;

    (&logging.lock().await)
        .chunks()
        .enumerate()
        .for_each(|(i, data)| {
            println!(
                "{:4}: {:5.0?}",
                i,
                data.iter().map(|x| x * 1e9).collect::<Vec<f64>>()
            );
        });

    let gom_ref = &mut (*gom.lock().await);
    let src = &mut (*gom_ref.src.lock().unwrap());
    let n = src.pupil_sampling();
    let _: complot::Heatmap = (
        (src.phase().as_slice(), (n, n)),
        Some(complot::Config::new().filename("opd.png")),
    )
        .into();

    Ok(())
}
