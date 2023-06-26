#![allow(unused_imports)]

use crseo::FromBuilder;
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{OneSignal, Signal, Signals, Smooth, Weight};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{
    fem_io::{
        actors_inputs::{MCM2Lcl6F, OSSM1Lcl6F, CFD2021106F},
        actors_outputs::OSSM1Lcl,
    },
    DiscreteModalSolver, ExponentialMatrix,
};
use gmt_dos_clients_io::{
    cfd_wind_loads::{CFDM1WindLoads, CFDM2WindLoads, CFDMountWindLoads},
    gmt_m2::asm::segment::{AsmCommand, FaceSheetFigure, VoiceCoilsForces, VoiceCoilsMotion},
};
use gmt_dos_clients_m1_ctrl::{Calibration as M1Calibration, Segment as M1Segment};
use gmt_dos_clients_m2_ctrl::Preprocessor;
use gmt_dos_clients_m2_ctrl::{Calibration as AsmsCalibration, Segment as AsmsSegment};
use gmt_dos_clients_mount::Mount;
use gmt_dos_clients_windloads::CfdLoads;
use gmt_fem::FEM;
use nalgebra::DMatrix;
use ngao_opm::{AsmsDispatch, Ngao};
use parse_monitors::cfd;
use polars::prelude::*;
use serde::Serialize;
use std::{
    env,
    fs::{DirBuilder, File},
    io::Write,
    path::Path,
    time::Duration,
};

const ACTUATOR_RATE: usize = 100;
const PYWFS: usize = 8;
const HDFS: usize = 800;

#[derive(Debug, Serialize)]
pub struct Settings {
    /// FEM path
    fem_repo: String,
    /// CEO GMT modes path
    gmt_modes_path: String,
    /// Results path
    data_repo: String,
    /// Karhunen-Loeve # of modes
    n_kl_mode: usize,
    #[cfg(feature = "domeseeing")]
    /// CFD cases path
    cfd_repo: Option<String>,
    #[cfg(feature = "domeseeing")]
    /// CFD zenith angle [deg]
    za: Option<usize>,
    #[cfg(feature = "domeseeing")]
    /// CFD azimith angle [deg]
    az: Option<usize>,
    #[cfg(feature = "domeseeing")]
    /// CFD vents/enclosdre configuration
    vs: Option<String>,
    #[cfg(feature = "domeseeing")]
    /// CFD wind speed
    ws: Option<usize>,
    /// Simulation duration [s]
    sim_duration: Option<usize>,
    /// Hankel singular values threshold
    hsv: Option<f64>,
}
impl Settings {
    pub fn from_env() -> Self {
        Self {
            fem_repo: env::var("FEM_REPO").unwrap(),
            gmt_modes_path: env::var("GMT_MODES_PATH").unwrap(),
            data_repo: env::var("DATA_REPO").unwrap(),
            n_kl_mode: env::var("N_KL_MODE")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap(),
            #[cfg(feature = "domeseeing")]
            cfd_repo: env::var("CFD_REPO").ok(),
            #[cfg(feature = "domeseeing")]
            za: env::var("ZA").ok().map(|v| v.parse::<usize>().unwrap()),
            #[cfg(feature = "domeseeing")]
            az: env::var("AZ").ok().map(|v| v.parse::<usize>().unwrap()),
            #[cfg(feature = "domeseeing")]
            vs: env::var("VS").ok(),
            #[cfg(feature = "domeseeing")]
            ws: env::var("WS").ok().map(|v| v.parse::<usize>().unwrap()),
            sim_duration: env::var("SIM_DURATION")
                .ok()
                .map(|v| v.parse::<usize>().unwrap()),
            hsv: env::var("HSV").ok().map(|v| v.parse::<f64>().unwrap()),
        }
    }
    pub fn save(self) {
        let toml_str = toml::to_string(&self).expect("Failed to serialize to TOML");
        let data_repo = env::var("DATA_REPO").unwrap();
        let path = Path::new(&data_repo);
        let mut file = File::create(path.join("settings.toml")).expect("Failed to create file");
        file.write_all(toml_str.as_bytes())
            .expect("Failed to write to file");
    }
}

pub fn set_cfd_case(za: usize, vw: &str, ws: usize) {
    env::set_var("ZA", format!("{za}"));
    env::set_var("AZ", "0");
    env::set_var("VW", format!("{vw}"));
    env::set_var("WS", format!("{ws}"));
}

pub fn cfd_lookup() {
    if let Ok(job_id) = env::var("AWS_BATCH_JOB_ARRAY_INDEX") {
        match job_id.parse::<usize>() {
            Ok(id) if id == 0 => set_cfd_case(0, "os", 2),
            Ok(id) if id == 1 => set_cfd_case(30, "os", 2),
            Ok(id) if id == 2 => set_cfd_case(60, "os", 2),
            Ok(id) if id == 3 => set_cfd_case(0, "os", 7),
            Ok(id) if id == 4 => set_cfd_case(30, "os", 7),
            Ok(id) if id == 5 => set_cfd_case(60, "os", 7),
            Ok(id) if id == 6 => set_cfd_case(0, "cd", 12),
            Ok(id) if id == 7 => set_cfd_case(30, "cd", 12),
            Ok(id) if id == 8 => set_cfd_case(60, "cs", 12),
            Ok(id) if id == 9 => set_cfd_case(0, "cd", 17),
            Ok(id) if id == 10 => set_cfd_case(30, "cd", 17),
            Ok(id) if id == 11 => set_cfd_case(60, "cs", 17),
            _ => eprintln!("AWS_BATCH_JOB_ARRAY_INDEX value is incorrect"),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    // let mut builder = env_logger::Builder::new();
    // builder
    //     .format_timestamp(None)
    //     .filter_module("gmt_dos-actors", log::LevelFilter::Debug);
    // builder.init();

    /*     let sim_sampling_frequency = 8000;
    let sim_duration = 1_usize; // second
    let n_step = sim_sampling_frequency * sim_duration; */
    let sim_sampling_frequency = 8_000usize; // Hz
    let (sim_duration, n_step) = if let Ok(sim_duration) = env::var("SIM_DURATION") {
        let sim_duration = sim_duration.parse::<usize>().unwrap();
        (sim_duration, sim_duration * sim_sampling_frequency)
    } else {
        let n_step = HDFS * 10; // sim_duration * sampling_frequency;
        let sim_duration = n_step / sim_sampling_frequency;
        (sim_duration, n_step)
    };
    dbg!((sim_duration, n_step));

    let mut fem = FEM::from_env()?;
    // println!("{fem}");

    #[cfg(feature = "domeseeing")]
    cfd_lookup();
    #[cfg(feature = "domeseeing")]
    let (za, az, vw, ws): (u32, u32, String, u32) = (
        env::var("ZA").map_or_else(|_| 30, |v| v.parse().unwrap()),
        env::var("AZ").map_or_else(|_| 0, |v| v.parse().unwrap()),
        env::var("VW").unwrap_or("os".to_string()),
        env::var("WS").map_or_else(|_| 7, |v| v.parse().unwrap()),
    );
    // CFD WIND LOADS
    #[cfg(feature = "domeseeing")]
    let cfd_repo = env::var("CFD_REPO").expect("CFD_REPO env var missing");
    #[cfg(feature = "domeseeing")]
    let cfd_case = cfd::CfdCase::<2021>::colloquial(za, az, &vw, ws)?;
    #[cfg(feature = "domeseeing")]
    println!("CFD CASE: {cfd_case}");
    #[cfg(feature = "domeseeing")]
    let cfd_path = Path::new(&cfd_repo).join(cfd_case.to_string());
    #[cfg(feature = "windloading")]
    let cfd_loads_client = CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency)
        .duration(sim_duration as f64)
        .mount(&mut fem, 0, None)
        .m1_segments()
        .m2_segments()
        .build()?
        .into_arcx();

    let timestamp = chrono::Local::now().to_rfc3339();

    #[cfg(all(not(feature = "domeseeing"), not(feature = "windloading")))]
    let data_repo = Path::new("/fsx")
        .join("ao4elt7")
        .join(timestamp)
        .join("atmosphere");
    #[cfg(all(feature = "domeseeing", not(feature = "windloading")))]
    let data_repo = Path::new("/fsx")
        .join("ao4elt7")
        .join(timestamp)
        .join("domeseeing");
    #[cfg(feature = "windloading")]
    let data_repo = Path::new("/fsx")
        .join("ao4elt7")
        .join(timestamp)
        .join("windloading");
    if !data_repo.is_dir() {
        DirBuilder::new().recursive(true).create(&data_repo)?;
    }
    env::set_var("DATA_REPO", &data_repo);

    Settings::from_env().save();

    let m1_calibration =
        if let Ok(m1_calibration) = M1Calibration::try_from(data_repo.join("m1_calibration.bin")) {
            m1_calibration
        } else {
            let m1_calibration = M1Calibration::new(&mut fem.clone());
            m1_calibration
                .save(data_repo.join("m1_calibration.bin"))
                .expect("failed to save M1 calibration");
            m1_calibration
        };

    let n_lenslet = 96;
    let n_px_lenslet = 8;

    let n_mode: usize = env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());
    let n_actuator = 675;

    let sids = vec![1, 2, 3, 4, 5, 6, 7];
    let calibration_file_name =
        data_repo.join(format!("asms_zonal_kl{n_mode}gs36_calibration.bin"));
    let mut asms_calibration = if let Ok(data) = AsmsCalibration::try_from(&calibration_file_name) {
        data
    } else {
        let asms_calibration = AsmsCalibration::builder(
            n_mode,
            n_actuator,
            (
                Path::new(&env::var("FEM_REPO").unwrap())
                    .join("KLmodesGS36.mat")
                    .to_str()
                    .unwrap()
                    .to_string(),
                (1..=7).map(|i| format!("KL_{i}")).collect::<Vec<String>>(),
            ),
            &mut fem.clone(),
        )
        .stiffness("Zonal")
        .build()?;
        asms_calibration.save(&calibration_file_name)?;
        asms_calibration
    };
    asms_calibration.transpose_modes();

    // let fem_file_name = data_repo.join(format!("fem_state-space_full_{n_mode}kl_cfd.bin"));
    // let fem_dss = if let Ok(fem_dss) =
    //     { DiscreteModalSolver::<ExponentialMatrix>::try_from(fem_file_name.clone()) }
    // {
    //     fem_dss
    // } else {
    let fem_dss = {
        let dss = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            // .truncate_hankel_singular_values(1e-5)
            // .hankel_frequency_lower_bound(50.)
            .ins::<CFD2021106F>()
            .ins::<OSSM1Lcl6F>()
            .ins::<MCM2Lcl6F>()
            .including_mount()
            .including_m1(Some(sids.clone()))?
            .including_asms(Some(sids.clone()), None, None)?
            .outs::<OSSM1Lcl>()
            .outs_with_by_name(
                sids.iter()
                    .map(|i| format!("M2_segment_{i}_axial_d"))
                    .collect::<Vec<_>>(),
                asms_calibration.modes_t(Some(sids.clone())).unwrap(),
            )
            .unwrap()
            .use_static_gain_compensation();
        let hsv = dss.hankel_singular_values()?;
        serde_pickle::to_writer(
            &mut File::create(data_repo.join("hsv.pkl"))?,
            &hsv,
            Default::default(),
        )?;
        if let Ok(Ok(hsv)) = env::var("HSV").and_then(|v| Ok(v.parse::<f64>())) {
            dss.truncate_hankel_singular_values(hsv).build()
        } else {
            dss.build()
        }?
    };
    //     fem_dss.save(fem_file_name)?;
    //     fem_dss
    // };
    println!("{fem_dss}");

    let mut plant: Actor<_> = Actor::new(fem_dss.into_arcx())
        /*         .name(format!(
                "GMT
        Finite Element Model
        {}",
                env::var("FEM_REPO").unwrap()
            )) */
        .name("")
        .image("fem.png");

    #[cfg(feature = "windloading")]
    let mut cfd_loads: Initiator<_> = Actor::new(cfd_loads_client.clone()).name("CFD Wind loads");
    #[cfg(feature = "windloading")]
    let mut signals = Signals::new(1, n_step).channel(
        0,
        Signal::Sigmoid {
            amplitude: 1f64,
            sampling_frequency_hz: sim_sampling_frequency as f64,
        },
    );
    #[cfg(feature = "windloading")]
    signals.progress();
    #[cfg(feature = "windloading")]
    let signal = OneSignal::try_from(signals)?.into_arcx();
    #[cfg(feature = "windloading")]
    let m1_smoother = Smooth::new().into_arcx();
    #[cfg(feature = "windloading")]
    let m2_smoother = Smooth::new().into_arcx();
    #[cfg(feature = "windloading")]
    let mount_smoother = Smooth::new().into_arcx();

    #[cfg(feature = "windloading")]
    let mut sigmoid: Initiator<_> = Actor::new(signal.clone()).name("Sigmoid");
    #[cfg(feature = "windloading")]
    let mut smooth_m1_loads: Actor<_> = Actor::new(m1_smoother.clone());
    #[cfg(feature = "windloading")]
    let mut smooth_m2_loads: Actor<_> = Actor::new(m2_smoother.clone());
    #[cfg(feature = "windloading")]
    let mut smooth_mount_loads: Actor<_> = Actor::new(mount_smoother.clone());

    #[cfg(feature = "windloading")]
    {
        sigmoid
            .add_output()
            .multiplex(3)
            .build::<Weight>()
            .into_input(&mut smooth_m1_loads)
            .into_input(&mut smooth_m2_loads)
            .into_input(&mut smooth_mount_loads)?;
        cfd_loads
            .add_output()
            .build::<CFDM1WindLoads>()
            .into_input(&mut smooth_m1_loads)?;
        smooth_m1_loads
            .add_output()
            .build::<CFDM1WindLoads>()
            .into_input(&mut plant)?;
        cfd_loads
            .add_output()
            .build::<CFDM2WindLoads>()
            .into_input(&mut smooth_m2_loads)?;
        smooth_m2_loads
            .add_output()
            .build::<CFDM2WindLoads>()
            .into_input(&mut plant)?;
        cfd_loads
            .add_output()
            .build::<CFDMountWindLoads>()
            .into_input(&mut smooth_mount_loads)?;
        smooth_mount_loads
            .add_output()
            .build::<CFDMountWindLoads>()
            .into_input(&mut plant)?;
    }
    // let plant_logging = Logging::<f64>::new(sids.len() + 1).into_arcx();
    // let mut plant_logger: Terminator<_> = Actor::new(plant_logging.clone());

    let file = File::open(Path::new(&env::var("FEM_REPO").unwrap()).join("ASMS-nodes.parquet"))?;
    let df = ParquetReader::new(file).finish()?;
    let nodes: Vec<_> = df["S7"]
        .iter()
        .filter_map(|series| {
            if let AnyValue::List(series) = series {
                series
                    .f64()
                    .ok()
                    .map(|x| x.into_iter().take(2).filter_map(|x| x).collect::<Vec<_>>())
            } else {
                None
            }
        })
        .flatten()
        .collect();
    let stiffness = nalgebra::DMatrix::<f64>::from_column_slice(
        n_actuator,
        n_actuator,
        asms_calibration.stiffness(7),
    );
    let p7_to_m7: Option<DMatrix<f64>> = asms_calibration
        .modes_t(Some(vec![7]))
        .and_then(|mut mat| mat.pop())
        .map(|mat| mat.into());
    dbg!(p7_to_m7.as_ref().map(|x| x.shape()));
    let prep = Preprocessor::new(nodes, stiffness.as_view(), p7_to_m7);

    let m: Vec<DMatrix<f64>> = asms_calibration
        .modes(Some(sids.clone()))
        .iter()
        .map(|x| x.clone_owned())
        .collect();
    let mut asms_dispatch: Actor<_, PYWFS, 1> =
        AsmsDispatch::new(n_mode, Some(m), Some(prep)).into();

    let fov = 0f32;
    #[cfg(not(feature = "domeseeing"))]
    let (gom, ngao_model) = Ngao::<PYWFS, HDFS>::builder()
        .n_lenslet(n_lenslet)
        .n_px_lenslet(n_px_lenslet)
        .modes_src_file("M2_OrthoNormGS36_KarhunenLoeveModes")
        .n_mode(n_mode)
        .gain(0.5)
        .wrapping(760e-9 * 0.5)
        // .piston_capture(PistonCapture::Bound(0.375 * 760e-9))
        .atmosphere(
            crseo::Atmosphere::builder().ray_tracing(
                25.5,
                512usize.max(n_lenslet * n_px_lenslet) as i32,
                fov,
                60f32.max(sim_duration as f32),
                Some(
                    Path::new(&env::var("GMT_MODES_PATH").unwrap())
                        .join("ngao_atmophere.bin")
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
                None,
            ),
        )
        .build(
            n_step,
            sim_sampling_frequency as f64,
            &mut asms_dispatch,
            &mut plant,
        )
        .await?;
    #[cfg(feature = "domeseeing")]
    let (gom, ngao_model) = Ngao::<PYWFS, HDFS>::builder()
        .n_lenslet(n_lenslet)
        .n_px_lenslet(n_px_lenslet)
        .modes_src_file("M2_OrthoNormGS36_KarhunenLoeveModes")
        .n_mode(n_mode)
        .gain(0.5)
        .wrapping(760e-9 * 0.5)
        // .piston_capture(PistonCapture::Bound(0.375 * 760e-9))
        .atmosphere(
            crseo::Atmosphere::builder().ray_tracing(
                25.5,
                512usize.max(n_lenslet * n_px_lenslet) as i32,
                fov,
                60f32.max(sim_duration as f32),
                Some(
                    Path::new(&env::var("GMT_MODES_PATH").unwrap())
                        .join("ngao_atmophere.bin")
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
                None,
            ),
        )
        .dome_seeing(cfd_path, sim_sampling_frequency / 5)
        .build(
            n_step,
            sim_sampling_frequency as f64,
            &mut asms_dispatch,
            &mut plant,
        )
        .await?;

    let mut asms_logger: Terminator<_> = (
        Arrow::builder(n_step).decimation(8).build(),
        "ASMS
Logger",
    )
        .into();

    let mut m1: Model<model::Unknown> = Default::default();
    let mut m2: Model<model::Unknown> = Default::default();
    let mut setpoints: Model<model::Unknown> = Default::default();
    for &sid in &sids {
        match sid {
            i if i == 1 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), "Setpoint").into();
                let mut actuators_setpoint: Initiator<_, ACTUATOR_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    "Setpoint",
                )
                    .into();
                m1 += M1Segment::<1, ACTUATOR_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                )
                .build(&mut plant)?
                .name("M1S1")
                .flowchart();
                setpoints += rbm_setpoint + actuators_setpoint;

                let mut asm_outer_segment = AsmsSegment::<1>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?;
                asm_outer_segment
                    .add_output()
                    .unbounded()
                    .build::<VoiceCoilsForces<1>>()
                    .logn(&mut asms_logger, 675)
                    .await?;
                m2 += model!(asm_outer_segment).name("M1S2").flowchart();
            }
            i if i == 2 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), "Setpoint").into();
                let mut actuators_setpoint: Initiator<_, ACTUATOR_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    "Setpoint",
                )
                    .into();
                m1 += M1Segment::<2, ACTUATOR_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                )
                .build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;

                m2 += AsmsSegment::<2>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?;
            }
            i if i == 3 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), "Setpoint").into();
                let mut actuators_setpoint: Initiator<_, ACTUATOR_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    "Setpoint",
                )
                    .into();
                m1 += M1Segment::<3, ACTUATOR_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                )
                .build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;

                m2 += AsmsSegment::<3>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?;
            }
            i if i == 4 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), "Setpoint").into();
                let mut actuators_setpoint: Initiator<_, ACTUATOR_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    "Setpoint",
                )
                    .into();
                m1 += M1Segment::<4, ACTUATOR_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                )
                .build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;

                m2 += AsmsSegment::<4>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?;
            }
            i if i == 5 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), "Setpoint").into();
                let mut actuators_setpoint: Initiator<_, ACTUATOR_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    "Setpoint",
                )
                    .into();
                m1 += M1Segment::<5, ACTUATOR_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                )
                .build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;

                m2 += AsmsSegment::<5>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?;
            }
            i if i == 6 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), "Setpoint").into();
                let mut actuators_setpoint: Initiator<_, ACTUATOR_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    "Setpoint",
                )
                    .into();
                m1 += M1Segment::<6, ACTUATOR_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                )
                .build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;

                m2 += AsmsSegment::<6>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?;
            }
            i if i == 7 => {
                let mut rbm_setpoint: Initiator<_> = (Signals::new(6, n_step), "Setpoint").into();
                let mut actuators_setpoint: Initiator<_, ACTUATOR_RATE> = (
                    Signals::new(if i == 7 { 306 } else { 335 }, n_step),
                    "Setpoint",
                )
                    .into();
                m1 += M1Segment::<7, ACTUATOR_RATE>::builder(
                    m1_calibration.clone(),
                    &mut rbm_setpoint,
                    &mut actuators_setpoint,
                )
                .build(&mut plant)?;
                setpoints += rbm_setpoint + actuators_setpoint;

                let mut asm_center_segment = AsmsSegment::<7>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?;
                asm_center_segment
                    .add_output()
                    .unbounded()
                    .build::<VoiceCoilsForces<7>>()
                    .logn(&mut asms_logger, 675)
                    .await?;
                m2 += asm_center_segment;
            }
            _ => unimplemented!("Segments ID must be in the range [1,7]"),
        }
    }

    plant
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<VoiceCoilsMotion<1>>()
        .logn(&mut asms_logger, n_actuator)
        .await?;
    plant
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<VoiceCoilsMotion<7>>()
        .logn(&mut asms_logger, n_actuator)
        .await?;
    plant
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<FaceSheetFigure<1>>()
        .logn(&mut asms_logger, n_mode)
        .await?;
    plant
        .add_output()
        .bootstrap()
        .unbounded()
        .build::<FaceSheetFigure<7>>()
        .logn(&mut asms_logger, n_mode)
        .await?;
    // let last_mode = env::args().nth(1).map_or_else(|| 1, |x| x.parse().unwrap());
    /*     let mut mode: Vec<usize> = (0..=dbg!(n_mode) - 1).collect();
    mode.dedup();
    let mut signals = mode
        .iter()
        .skip(1)
        .fold(Signals::new(n_mode, n_step), |s, i| {
            s.channel(
                *i,
                Signal::white_noise()
                    .expect("fishy!")
                    .std_dev(1e-7)
                    .expect("very fishy!"),
            )
        });
    let mut asm_setpoint: Initiator<Signals, 1> = (signals, "White Noise").into();
    asm_setpoint
        .add_output()
        .build::<AsmCommand<1>>()
        .into_input(&mut asms_dispatch)?; */

    // MOUNT CONTROL
    let mut mount_setpoint: Initiator<_> = (
        Signals::new(3, n_step),
        "Mount
    Setpoint",
    )
        .into();
    let mount_signal = mount_setpoint.client();
    let mount: Actor<_> = Mount::builder(&mut mount_setpoint).build(&mut plant)?;
    setpoints += mount_setpoint;

    /*     plant
    .add_output()
    .bootstrap()
    .build::<M1RigidBodyMotions>()
    .into_input(&mut plant_logger)?; */

    #[cfg(feature = "windloading")]
    let cfd_input_model = model!(
        cfd_loads,
        sigmoid,
        smooth_m1_loads,
        smooth_m2_loads,
        smooth_mount_loads
    );
    #[cfg(feature = "windloading")]
    let model = (model!(plant)
        + cfd_input_model
        + mount
        + m1
        + m2
        + setpoints
        + ngao_model
        + asms_dispatch
        + asms_logger)
        .name("ngao-opm")
        .flowchart()
        .check()?
        .run();
    #[cfg(not(feature = "windloading"))]
    let model =
        (model!(plant) + mount + m1 + m2 + setpoints + ngao_model + asms_dispatch + asms_logger)
            .name("ngao-opm")
            .flowchart()
            .check()?
            .run();
    std::thread::sleep(Duration::from_secs(3));
    (&mut *mount_signal.lock().await).progress();
    model.await?;

    let gom = &mut (*gom.lock().await);
    let src = &mut (*gom.src.lock().unwrap());
    let n = src.pupil_sampling();
    let opd: Vec<_> = src.phase().iter().map(|x| *x * 1e6).collect();
    let mut file = File::create(data_repo.join("opd.pkl"))?;
    serde_pickle::to_writer(&mut file, &opd, Default::default())?;

    #[cfg(feature = "complot")]
    {
        let _: complot::Heatmap = (
            (opd.as_slice(), (n, n)),
            Some(
                complot::Config::new()
                    .filename(data_repo.join("opd.png").to_str().unwrap().to_string()),
            ),
        )
            .into();
    }

    Ok(())
}
