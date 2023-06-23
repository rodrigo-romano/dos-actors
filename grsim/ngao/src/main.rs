use std::{env, path::Path, time::Instant};

use crseo::{
    wavefrontsensor::{PhaseSensor, PistonSensor, SegmentCalibration},
    Builder, FromBuilder, Gmt, SegmentWiseSensorBuilder, WavefrontSensorBuilder,
};
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Logging, Pulse, Sampler, Tick, Timer};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_crseo::{M2modes, SegmentPiston, SegmentWfeRms, WfeRms};
use ngao::{
    GuideStar, LittleOpticalModel, PwfsIntegrator, ResidualM2modes, ResidualPistonMode, SensorData,
    WavefrontSensor,
};

const PYWFS_READOUT: usize = 8;
const PYWFS: usize = 8;
const HDFS: usize = 800;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let data_repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    dbg!(&data_repo);
    env::set_var("DATA_REPO", &data_repo);
    env::set_var("GMT_MODES_PATH", &data_repo);

    let sampling_frequency = 8_000usize; // Hz
    let sim_duration = 1usize;
    let n_sample = HDFS * 10; // sim_duration * sampling_frequency;

    // assert_eq!(sampling_frequency / PYWFS_READOUT, 4000);
    // assert_eq!(sampling_frequency / PYWFS, 4000);

    let n_lenslet = 92;
    let n_mode: usize = env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());

    let builder = PhaseSensor::builder()
        .lenslet(n_lenslet, 4)
        .wrapping(760e-9 * 0.5);
    let src_builder = builder.guide_stars(None);

    let m2_modes = "M2_OrthoNormGS36_KarhunenLoeveModes";
    // let m2_modes = "Karhunen-Loeve";

    let now = Instant::now();
    let mut slopes_mat = builder.clone().calibrate(
        SegmentCalibration::modes(m2_modes, 0..n_mode, "M2"),
        src_builder.clone(),
    );
    println!(
        "M2 {}modes/segment calibrated in {}s",
        n_mode,
        now.elapsed().as_secs()
    );
    slopes_mat.pseudo_inverse(None).unwrap();

    let piston_builder = PistonSensor::builder().pupil_sampling(builder.pupil_sampling());
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
    let p2m = piston_mat.concat_pinv();
    dbg!(&p2m);

    let gom = LittleOpticalModel::builder()
        .gmt(Gmt::builder().m2(m2_modes, n_mode))
        .source(src_builder)
        .atmosphere(
            crseo::Atmosphere::builder().ray_tracing(
                25.5,
                builder.pupil_sampling() as i32,
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
        .build()?
        .into_arcx();

    let mut gom_act: Actor<_> = Actor::new(gom.clone()).name("GS>>(GMT+ATM)");

    let mut sensor: Actor<_, 1, PYWFS_READOUT> =
        (WavefrontSensor::new(builder.build()?, slopes_mat), "PWFS").into();
    let mut piston_sensor: Actor<_, 1, HDFS> = (
        WavefrontSensor::new(piston_builder.build()?, piston_mat),
        "HDFS",
    )
        .into();

    let mut timer: Initiator<Timer, 1> = Timer::new(n_sample).into();

    // let logging = Logging::new(2).into_arcx();
    let logging = Arrow::builder(n_sample)
        .filename("ngao.parquet")
        .build()
        .into_arcx();
    let mut logger: Terminator<_> = Actor::new(logging.clone());
    let piston_logging = Logging::new(1).into_arcx();
    let mut piston_logger: Terminator<_, HDFS> = Actor::new(piston_logging.clone()).name(
        "HDFS
    Logger",
    );

    let mut sampler_hdfs_to_pwfs: Actor<_, HDFS, PYWFS> = (
        Pulse::new(1),
        "Rate transition:
    PWFS -> HDFS",
    )
        .into();
    /*     let mut sampler_pwfs_to_hdfs: Actor<_, PYWFS, HDFS> = (
        Sampler::new(vec![0f64; 7]),
        "Rate transition:
    PWFS -> HDFS",
    )
        .into(); */
    let mut sampler_pwfs_to_plant: Actor<_, PYWFS, 1> = (
        Sampler::default(),
        "Rate transition:
    PWFS -> ASMS",
    )
        .into();

    // let b = 0.375 * 760e-9;
    // let b = f64::INFINITY; // PISTON PWFS
    // let b = f64::NEG_INFINITY; // PISTON HDFS
    /*     let mut hdfs_integrator: Actor<_, HDFS, PYWFS> = (
        HdfsIntegrator::new(0.5f64, p2m, b),
        "HDFS
    Integrator",
    )
        .into(); */
    let mut pwfs_integrator: Actor<_, PYWFS, PYWFS> = (
        PwfsIntegrator::single_single(n_mode, 0.5f64),
        "PWFS
    Integrator",
    )
        .into();

    timer
        .add_output()
        .build::<Tick>()
        .into_input(&mut gom_act)?;
    gom_act
        .add_output()
        .multiplex(2)
        .build::<GuideStar>()
        .into_input(&mut sensor)
        .into_input(&mut piston_sensor)?;
    sensor
        .add_output()
        .build::<ResidualM2modes>()
        .into_input(&mut pwfs_integrator)?;

    /*     sampler_pwfs_to_pwfs_ctrl
    .add_output()
    .bootstrap()
    .build::<ResidualM2modes>()
    .into_input(&mut pwfs_integrator)?; */
    gom_act
        .add_output()
        .unbounded()
        .build::<WfeRms>()
        .log(&mut logger)
        .await?;
    gom_act
        .add_output()
        .unbounded()
        .build::<SegmentWfeRms>()
        .log(&mut logger)
        .await?;
    gom_act
        .add_output()
        .unbounded()
        .build::<SegmentPiston>()
        .log(&mut logger)
        .await?;
    piston_sensor
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<SensorData>()
        .into_input(&mut piston_logger)?;
    piston_sensor
        .add_output()
        .bootstrap()
        .build::<ResidualPistonMode>()
        .into_input(&mut sampler_hdfs_to_pwfs)?;
    sampler_hdfs_to_pwfs
        .add_output()
        // .bootstrap()
        .build::<ResidualPistonMode>()
        .into_input(&mut pwfs_integrator)?;
    pwfs_integrator
        .add_output()
        .bootstrap()
        .build::<M2modes>()
        .into_input(&mut sampler_pwfs_to_plant)?;
    sampler_pwfs_to_plant
        .add_output()
        .build::<M2modes>()
        .into_input(&mut gom_act)?;

    model!(
        timer,
        gom_act,
        sensor,
        piston_sensor,
        logger,
        piston_logger,
        // hdfs_integrator,
        pwfs_integrator,
        sampler_hdfs_to_pwfs,
        sampler_pwfs_to_plant
    )
    .name("NGAO")
    .flowchart()
    .check()?
    .run()
    .await?;

    /*     let n_show = 10;
    (&logging.lock().await)
        .chunks()
        .enumerate()
        .skip(n_sample - n_show)
        .for_each(|(i, data)| {
            println!(
                "{:4}: {:5.0?}",
                i,
                data.iter().map(|x| x * 1e9).collect::<Vec<f64>>()
            );
        });
    (&logging.lock().await).to_mat_file("ngao.mat")?;

    (&piston_logging.lock().await)
        .chunks()
        .enumerate()
        .skip(n_sample / HDFS - n_show)
        .for_each(|(i, data)| {
            println!(
                "{:4}: {:5.0?}",
                i,
                data.iter().map(|x| x * 1e9).collect::<Vec<f32>>()
            );
        });
    (&piston_logging.lock().await).to_mat_file("hdfs.mat")?; */

    #[cfg(features = "complot")]
    {
        let gom_ref = &mut (*gom.lock().await);
        let src = &mut (*gom_ref.src.lock().unwrap());
        let n = src.pupil_sampling();
        let _: complot::Heatmap = (
            (src.phase().as_slice(), (n, n)),
            Some(
                complot::Config::new()
                    .filename(data_repo.join("opd.png").to_str().unwrap().to_string()),
            ),
        )
            .into();
    }
    Ok(())
}
