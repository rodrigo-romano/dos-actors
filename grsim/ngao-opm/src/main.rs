use crseo::FromBuilder;
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::Signals;
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_m1_ctrl::{Calibration as M1Calibration, Segment as M1Segment};
use gmt_dos_clients_m2_ctrl::{Calibration as AsmsCalibration, Segment as AsmsSegment};
use gmt_dos_clients_mount::Mount;
use gmt_fem::{fem_io::OSSM1Lcl, FEM};
use ngao_opm::{AsmsDispatch, Ngao};
use std::{env, path::Path};

const ACTUATOR_RATE: usize = 100;
const PYWFS: usize = 8;
const HDFS: usize = 800;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let data_repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    env::set_var("DATA_REPO", &data_repo);
    env::set_var("GMT_MODES_PATH", &data_repo);

    /*     let sim_sampling_frequency = 8000;
    let sim_duration = 1_usize; // second
    let n_step = sim_sampling_frequency * sim_duration; */
    let sim_sampling_frequency = 8_000usize; // Hz
    let n_step = HDFS * 10; // sim_duration * sampling_frequency;
    let sim_duration = n_step / sim_sampling_frequency;

    let mut fem = FEM::from_env()?;
    println!("{fem}");
    let m1_calibration = M1Calibration::new(&mut fem);

    let n_lenslet = 92;
    let n_mode: usize = env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());
    let n_actuator = 675;

    let sids = vec![1, 2, 3, 4, 5, 6, 7];
    let calibration_file_name = Path::new(".").join(format!("asms_kl{n_mode}_calibration.bin"));
    let mut asms_calibration = if let Ok(data) = AsmsCalibration::load(&calibration_file_name) {
        data
    } else {
        let asms_calibration = AsmsCalibration::new(
            n_mode,
            n_actuator,
            (
                "data/KLmodes.mat".to_string(),
                (1..=7).map(|i| format!("KL_{i}")).collect::<Vec<String>>(),
            ),
            &mut fem,
        )?;
        asms_calibration.save(&calibration_file_name)?;
        AsmsCalibration::load(calibration_file_name)?
    };
    asms_calibration.transpose_modes();

    let fem_dss = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        // .truncate_hankel_singular_values(1e-5)
        // .hankel_frequency_lower_bound(50.)
        .including_mount()
        .including_m1(Some(sids.clone()))?
        .including_asms(asms_calibration.modes(Some(sids.clone())), asms_calibration.modes_t(Some(sids.clone()))
        .expect(r#"expect some transposed modes, found none (have you called "Calibration::transpose_modes"#), Some(sids.clone()))?
        .outs::<OSSM1Lcl>()
        .outs_with_by_name(sids.iter().map(|i| format!("M2_segment_{i}_axial_d")).collect::<Vec<_>>(),
         asms_calibration.modes_t(Some(sids.clone())).unwrap()).unwrap()
        .use_static_gain_compensation()
        .build()?;
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

    // let plant_logging = Logging::<f64>::new(sids.len() + 1).into_arcx();
    // let mut plant_logger: Terminator<_> = Actor::new(plant_logging.clone());

    let mut asms_dispatch: Actor<_> = AsmsDispatch::new(n_mode).into();

    let n_px_lenslet = 4;
    let fov = 0f32;
    let (gom, ngao_model) = Ngao::<PYWFS, HDFS>::builder()
        .n_lenslet(n_lenslet)
        .n_px_lenslet(4)
        .modes_src_file("M2_OrthoNorm_KarhunenLoeveModes")
        .n_mode(n_mode)
        // .wrapping(760e-9 * 0.5)
        // .piston_capture(PistonCapture::Bound(0.375 * 760e-9))
        .atmosphere(
            crseo::Atmosphere::builder().ray_tracing(
                25.5,
                512usize.max(n_lenslet * n_px_lenslet) as i32,
                fov,
                1f32.max(sim_duration as f32),
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
        .build(
            n_step,
            sim_sampling_frequency as f64,
            &mut asms_dispatch,
            &mut plant,
        )
        .await?;

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

                m2 += model!(AsmsSegment::<1>::builder(
                    n_mode,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?)
                .name("M1S2")
                .flowchart();
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
                    n_mode,
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
                    n_mode,
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
                    n_mode,
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
                    n_mode,
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
                    n_mode,
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

                m2 += AsmsSegment::<7>::builder(
                    n_mode,
                    asms_calibration.stiffness(i),
                    &mut asms_dispatch,
                )
                .build(&mut plant)?;
            }
            _ => unimplemented!("Segments ID must be in the range [1,7]"),
        }
    }

    // MOUNT CONTROL
    let mut mount_setpoint: Initiator<_> = (
        Signals::new(3, n_step),
        "Mount
    Setpoint",
    )
        .into();
    let mount: Actor<_> = Mount::builder(&mut mount_setpoint).build(&mut plant)?;
    setpoints += mount_setpoint;

    /*     plant
    .add_output()
    .bootstrap()
    .build::<M1RigidBodyMotions>()
    .into_input(&mut plant_logger)?; */

    (model!(plant) + mount + m1 + m2 + setpoints + ngao_model + asms_dispatch)
        .name("ngao-opm")
        .flowchart()
        .check()?
        .run()
        .await?;

    let gom = &mut (*gom.lock().await);
    let src = &mut (*gom.src.lock().unwrap());
    let n = src.pupil_sampling();
    let opd: Vec<_> = src.phase().iter().map(|x| *x * 1e6).collect();
    let _: complot::Heatmap = (
        (opd.as_slice(), (n, n)),
        Some(
            complot::Config::new()
                .filename(data_repo.join("opd.png").to_str().unwrap().to_string()),
        ),
    )
        .into();

    Ok(())
}
