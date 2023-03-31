use std::{env, path::Path, time::Instant};

use crseo::{
    wavefrontsensor::{
        DifferentialPistonSensor, GeomShack, PhaseSensor, PistonSensor, SegmentCalibration,
    },
    Builder, FromBuilder, Gmt, SegmentWiseSensorBuilder, Source, WavefrontSensorBuilder,
};
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Logging, Pulse, Sampler, Signal, Signals, Tick, Timer};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_crseo::{M2modes, SegmentPiston, SegmentWfeRms, WfeRms};
use matio_rs::MatFile;
use ngao::{
    GuideStar, LittleOpticalModel, M1Rxy, PwfsIntegrator, ResidualM2modes, ResidualPistonMode,
    SensorData, WavefrontSensor,
};
use skyangle::Conversion;

const PYWFS_READOUT: usize = 8;
const PYWFS: usize = 8;
const HDFS: usize = 800;
const AGWS: usize = 8_000;

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

    // _________________________________
    // NGAO CALIBRATION
    let n_lenslet = 92;
    let n_mode: usize = env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());

    let builder = PhaseSensor::builder()
        .lenslet(n_lenslet, 4)
        .wrapping(760e-9 * 0.5);
    let ngs_builder = builder.guide_stars(None);

    let m2_modes = "M2_OrthoNorm_KarhunenLoeveModes";
    // let m2_modes = "Karhunen-Loeve";

    let now = Instant::now();
    let mut slopes_mat = builder.clone().calibrate(
        SegmentCalibration::modes(m2_modes, 0..n_mode, "M2"),
        ngs_builder.clone(),
    );
    println!(
        "M2 {}modes/segment calibrated in {}s",
        n_mode,
        now.elapsed().as_secs()
    );
    slopes_mat.pseudo_inverse().unwrap();

    let piston_builder = PistonSensor::builder().pupil_sampling(builder.pupil_sampling());
    let now = Instant::now();
    let mut piston_mat = piston_builder.calibrate(
        SegmentCalibration::modes(m2_modes, 0..1, "M2"),
        ngs_builder.clone(),
    );
    println!(
        "M2 {}modes/segment calibrated in {}s",
        1,
        now.elapsed().as_secs()
    );
    piston_mat.pseudo_inverse().unwrap();
    let p2m = piston_mat.concat_pinv();
    dbg!(&p2m);
    // _________________________________

    // ACO MODEL
    let fov = 6f32.from_arcmin();
    let agws_sh48_builder = GeomShack::builder().size(3).lenslet(48, 8);
    let agws_gs_builder =
        agws_sh48_builder.guide_stars(Some(Source::builder().size(3).on_ring(fov / 2f32)));
    let dfs_builder =
        DifferentialPistonSensor::builder().pupil_sampling(agws_sh48_builder.pupil_sampling());

    let matfile = MatFile::save(data_repo.join("active-optics_calibrations.mat"))?;
    let now = Instant::now();
    let mut agws_sh48_rbm_calibration = agws_sh48_builder.clone().calibrate(
        SegmentCalibration::rbm("TRxyz", "M1"),
        agws_gs_builder.clone(),
    );
    println!(
        "M1 {}RBMs/segment calibrated in {}s",
        42,
        now.elapsed().as_secs()
    );
    for (i, mat) in agws_sh48_rbm_calibration
        .interaction_matrices()
        .iter()
        .enumerate()
    {
        matfile.var(format!("sh48_rbm{}", i + 1), mat)?;
    }
    agws_sh48_rbm_calibration.pseudo_inverse().unwrap();

    let now = Instant::now();
    let mut agws_sh48_bm_calibration = agws_sh48_builder.clone().calibrate(
        SegmentCalibration::modes("bending modes", 0..27, "M1"),
        agws_gs_builder.clone(),
    );
    println!(
        "M1 {}RBMs/segment calibrated in {}s",
        27,
        now.elapsed().as_secs()
    );
    for (i, mat) in agws_sh48_bm_calibration
        .interaction_matrices()
        .iter()
        .enumerate()
    {
        matfile.var(format!("sh48_bm{}", i + 1), mat)?;
    }
    agws_sh48_bm_calibration.pseudo_inverse().unwrap();

    let now = Instant::now();
    let mut dfs_calibration = dfs_builder.calibrate(
        SegmentCalibration::rbm("TRxyz", "M1"),
        agws_gs_builder.clone(),
    );
    for (i, mat) in dfs_calibration.interaction_matrices().iter().enumerate() {
        matfile.var(format!("dfs_rbm{}", i + 1), mat)?;
    }
    println!(
        "M1 {}RBMs/segment calibrated in {}s",
        42,
        now.elapsed().as_secs()
    );
    dfs_calibration.pseudo_inverse().unwrap();

    /*     let atmosphere_builder = crseo::Atmosphere::builder().ray_tracing(
        25.5,
        769,
        fov,
        sim_duration as f32,
        Some(
            data_repo
                .join("active-optics_atmosphere.bin")
                .to_str()
                .unwrap()
                .to_string(),
        ),
        None,
    ); */

    // NGAO MODEL
    let gmt_builder = Gmt::builder().m1("bending modes", 27).m2(m2_modes, n_mode);
    let ngao = LittleOpticalModel::builder()
        .gmt(gmt_builder.clone())
        .source(ngs_builder)
        // .atmosphere(atmosphere_builder.clone())
        .sampling_frequency(sampling_frequency as f64)
        .build()?
        .into_arcx();

    let agws = LittleOpticalModel::builder()
        .gmt(gmt_builder)
        .source(agws_gs_builder)
        // .atmosphere(atmosphere_builder)
        .sampling_frequency(sampling_frequency as f64)
        .build()?
        .into_arcx();
    let agws_logging = Arrow::builder(n_sample)
        .filename("agws.parquet")
        .build()
        .into_arcx();

    // MODEL
    let mut m1_rxy: Initiator<_> = (
        Signals::new(14, n_sample).channels(Signal::Constant(1e-6)),
        "M1 Rx & Ry",
    )
        .into();

    let mut ngao_act: Actor<_> = Actor::new(ngao.clone()).name(
        "ON-AXIS NGS
            >> (GMT+ATM)",
    );
    let mut agws_act: Actor<_> = Actor::new(agws).name(
        "AGWS 3 GS
            >> (GMT+ATM)",
    );
    let mut agws_logger: Terminator<_> = Actor::new(agws_logging.clone()).name("AGWS GS Logger");

    let mut pwfs: Actor<_, 1, PYWFS_READOUT> = (
        WavefrontSensor::new(builder.build()?, slopes_mat.clone()),
        "PWFS",
    )
        .into();
    let mut hdfs: Actor<_, 1, HDFS> = (
        WavefrontSensor::new(piston_builder.build()?, piston_mat.clone()),
        "HDFS",
    )
        .into();

    let mut agws_sh48: Terminator<_, 1> = (
        WavefrontSensor::new(agws_sh48_builder.build()?, agws_sh48_rbm_calibration),
        "AGWS SH48x3",
    )
        .into();
    let mut agws_dfs: Terminator<_, 1> = (
        WavefrontSensor::new(dfs_builder.build()?, dfs_calibration),
        "DFS",
    )
        .into();

    let mut timer: Initiator<_> = Timer::new(n_sample).into();

    // let logging = Logging::new(2).into_arcx();
    let logging = Arrow::builder(n_sample)
        .filename("ngao.parquet")
        .build()
        .into_arcx();
    let mut logger: Terminator<_> = Actor::new(logging.clone()).name("NGS Logger");
    let piston_logging = Logging::new(1).into_arcx();
    let mut piston_logger: Terminator<_, HDFS> = Actor::new(piston_logging.clone()).name(
        "HDFS
    Logger",
    );

    let mut sampler_hdfs_to_pwfs: Actor<_, HDFS, PYWFS> = (
        Pulse::new(1),
        "Pulse Transition:
    HDFS -> PWFS",
    )
        .into();

    let mut sampler_pwfs_to_plant: Actor<_, PYWFS, 1> = (
        Sampler::default(),
        "ZOH Transition:
    PWFS -> ASMS",
    )
        .into();

    let mut pwfs_integrator: Actor<_, PYWFS, PYWFS> = (
        PwfsIntegrator::single_single(n_mode, 0.5f64),
        "PWFS
    Integrator",
    )
        .into();

    m1_rxy
        .add_output()
        .multiplex(2)
        .build::<M1Rxy>()
        .into_input(&mut ngao_act)
        .into_input(&mut agws_act)?;
    timer
        .add_output()
        .build::<Tick>()
        .into_input(&mut ngao_act)?;
    ngao_act
        .add_output()
        .multiplex(2)
        .build::<GuideStar>()
        .into_input(&mut pwfs)
        .into_input(&mut hdfs)?;
    pwfs.add_output()
        .build::<ResidualM2modes>()
        .into_input(&mut pwfs_integrator)?;
    ngao_act
        .add_output()
        .unbounded()
        .build::<WfeRms>()
        .log(&mut logger)
        .await?;
    ngao_act
        .add_output()
        .unbounded()
        .build::<SegmentWfeRms>()
        .log(&mut logger)
        .await?;
    ngao_act
        .add_output()
        .unbounded()
        .build::<SegmentPiston>()
        .log(&mut logger)
        .await?;
    hdfs.add_output()
        .bootstrap()
        .unbounded()
        .build::<SensorData>()
        .into_input(&mut piston_logger)?;
    hdfs.add_output()
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
        .multiplex(2)
        .build::<M2modes>()
        .into_input(&mut ngao_act)
        .into_input(&mut agws_act)?;
    agws_act
        .add_output()
        .unbounded()
        .build::<WfeRms>()
        .log(&mut agws_logger)
        .await?;
    agws_act
        .add_output()
        .unbounded()
        .build::<SegmentWfeRms>()
        .log(&mut agws_logger)
        .await?;

    agws_act
        .add_output()
        .unbounded()
        .build::<SegmentPiston>()
        .log(&mut agws_logger)
        .await?;
    agws_act
        .add_output()
        .multiplex(2)
        .build::<GuideStar>()
        .into_input(&mut agws_sh48)
        .into_input(&mut agws_dfs)?;

    model!(
        timer,
        ngao_act,
        pwfs,
        hdfs,
        logger,
        piston_logger,
        pwfs_integrator,
        sampler_hdfs_to_pwfs,
        sampler_pwfs_to_plant,
        agws_logger,
        agws_act,
        agws_sh48,
        agws_dfs,
        m1_rxy
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

    let gom_ref = &mut (*ngao.lock().await);
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

    Ok(())
}
