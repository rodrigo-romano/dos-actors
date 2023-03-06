use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Logging, Signals};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::VoiceCoilsMotion;
use gmt_dos_clients_m2_ctrl::{segment, Calibration, Segment};
use gmt_fem::FEM;
use matio_rs::MatFile;

#[tokio::test]
async fn asms() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 8000;
    let sim_duration = 1_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mut fem = FEM::from_env()?;
    // whole_fem.keep_input::<>()
    // println!("{fem}");

    let n_mode = 66;
    let n_actuator = 675;

    let sids = vec![1, 2, 3, 4, 5, 6, 7];
    let mut asms_calibration = if let Ok(data) = Calibration::load("asms_stiffness.bin") {
        data
    } else {
        let asms_calibration = Calibration::new(
            n_mode,
            n_actuator,
            (
                "calib_dt/m2asm_ctrl_dt.mat".to_string(),
                (1..=7).map(|i| format!("V_S{i}")).collect::<Vec<String>>(),
            ),
            &mut fem,
        )?;
        asms_calibration.save("asms_stiffness.bin")?;
        Calibration::load("asms_stiffness.bin")?
    };
    asms_calibration.transpose_modes();

    let fem_as_state_space = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        .truncate_hankel_singular_values(4.855e-5)
        .hankel_frequency_lower_bound(50.)
        .including_asms(asms_calibration.modes(Some(sids.clone())), asms_calibration.modes_t(Some(sids.clone()))
        .expect(r#"expect some transposed modes, found none (have you call "Calibration::transpose_modes"#), Some(sids.clone()))?
        .build()?;
    println!("{fem_as_state_space}");
    let mut plant: Actor<_> = (fem_as_state_space, "Plant").into();

    let plant_logging = Logging::<f64>::new(sids.len()).into_arcx();
    let mut plant_logger: Terminator<_> = Actor::new(plant_logging.clone());

    let mut m2: Model<model::Unknown> = Default::default();
    let mut setpoints: Model<model::Unknown> = Default::default();

    for &sid in &sids {
        match sid {
            i if i == 1 => {
                let mut asm_setpoint: Initiator<_> = (
                    Signals::new(n_mode, n_step)
                        .channel(0, gmt_dos_clients::Signal::Constant(i as f64 * 1e-7))
                        .channel(
                            n_mode - 1,
                            gmt_dos_clients::Signal::Constant(i as f64 * 1e-7),
                        ),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 +=
                    Segment::<1>::builder(n_mode, asms_calibration.stiffness(i), &mut asm_setpoint)
                        .build(&mut plant)?;
                setpoints += asm_setpoint;
                plant
                    .add_output()
                    .bootstrap()
                    .build::<VoiceCoilsMotion<1>>()
                    .into_input(&mut plant_logger)?;
            }
            i if i == 2 => {
                let mut asm_setpoint: Initiator<_> = (
                    Signals::new(n_mode, n_step)
                        .channel(0, gmt_dos_clients::Signal::Constant(i as f64 * 1e-7))
                        .channel(
                            n_mode - 1,
                            gmt_dos_clients::Signal::Constant(i as f64 * 1e-7),
                        ),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 +=
                    Segment::<2>::builder(n_mode, asms_calibration.stiffness(i), &mut asm_setpoint)
                        .build(&mut plant)?;
                setpoints += asm_setpoint;
                plant
                    .add_output()
                    .bootstrap()
                    .build::<VoiceCoilsMotion<2>>()
                    .into_input(&mut plant_logger)?;
            }
            i if i == 3 => {
                let mut asm_setpoint: Initiator<_> = (
                    Signals::new(n_mode, n_step)
                        .channel(0, gmt_dos_clients::Signal::Constant(i as f64 * 1e-7))
                        .channel(
                            n_mode - 1,
                            gmt_dos_clients::Signal::Constant(i as f64 * 1e-7),
                        ),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 +=
                    Segment::<3>::builder(n_mode, asms_calibration.stiffness(i), &mut asm_setpoint)
                        .build(&mut plant)?;
                setpoints += asm_setpoint;
                plant
                    .add_output()
                    .bootstrap()
                    .build::<VoiceCoilsMotion<3>>()
                    .into_input(&mut plant_logger)?;
            }
            i if i == 4 => {
                let mut asm_setpoint: Initiator<_> = (
                    Signals::new(n_mode, n_step)
                        .channel(0, gmt_dos_clients::Signal::Constant(i as f64 * 1e-7))
                        .channel(
                            n_mode - 1,
                            gmt_dos_clients::Signal::Constant(i as f64 * 1e-7),
                        ),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 +=
                    Segment::<4>::builder(n_mode, asms_calibration.stiffness(i), &mut asm_setpoint)
                        .build(&mut plant)?;
                setpoints += asm_setpoint;
                plant
                    .add_output()
                    .bootstrap()
                    .build::<VoiceCoilsMotion<4>>()
                    .into_input(&mut plant_logger)?;
            }
            i if i == 5 => {
                let mut asm_setpoint: Initiator<_> = (
                    Signals::new(n_mode, n_step)
                        .channel(0, gmt_dos_clients::Signal::Constant(i as f64 * 1e-7))
                        .channel(
                            n_mode - 1,
                            gmt_dos_clients::Signal::Constant(i as f64 * 1e-7),
                        ),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 +=
                    Segment::<5>::builder(n_mode, asms_calibration.stiffness(i), &mut asm_setpoint)
                        .build(&mut plant)?;
                setpoints += asm_setpoint;
                plant
                    .add_output()
                    .bootstrap()
                    .build::<VoiceCoilsMotion<5>>()
                    .into_input(&mut plant_logger)?;
            }
            i if i == 6 => {
                let mut asm_setpoint: Initiator<_> = (
                    Signals::new(n_mode, n_step)
                        .channel(0, gmt_dos_clients::Signal::Constant(i as f64 * 1e-7))
                        .channel(
                            n_mode - 1,
                            gmt_dos_clients::Signal::Constant(i as f64 * 1e-7),
                        ),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 +=
                    Segment::<6>::builder(n_mode, asms_calibration.stiffness(i), &mut asm_setpoint)
                        .build(&mut plant)?;
                setpoints += asm_setpoint;
                plant
                    .add_output()
                    .bootstrap()
                    .build::<VoiceCoilsMotion<6>>()
                    .into_input(&mut plant_logger)?;
            }
            i if i == 7 => {
                let mut asm_setpoint: Initiator<_> = (
                    Signals::new(n_mode, n_step)
                        .channel(0, gmt_dos_clients::Signal::Constant(i as f64 * 1e-7))
                        .channel(
                            n_mode - 1,
                            gmt_dos_clients::Signal::Constant(i as f64 * 1e-7),
                        ),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 +=
                    Segment::<7>::builder(n_mode, asms_calibration.stiffness(i), &mut asm_setpoint)
                        .build(&mut plant)?;
                setpoints += asm_setpoint;
                plant
                    .add_output()
                    .bootstrap()
                    .build::<VoiceCoilsMotion<7>>()
                    .into_input(&mut plant_logger)?;
            }
            __ => unimplemented!(),
        }
    }

    (model!(plant, plant_logger) + setpoints + m2)
        .name("ASM_segment")
        .flowchart()
        .check()?
        .run()
        .await?;

    let n = sids.len();
    println!("{}", *plant_logging.lock().await);
    (*plant_logging.lock().await)
        .chunks()
        .enumerate()
        .skip(n_step - 21)
        .map(|(i, x)| {
            (
                i,
                x.iter()
                    .step_by(n_mode)
                    .take(n)
                    .map(|x| x * 1e7)
                    .collect::<Vec<f64>>(),
                x.iter()
                    .skip(n_mode - 1)
                    .step_by(n_mode)
                    .take(n)
                    .map(|x| x * 1e7)
                    .collect::<Vec<f64>>(),
            )
        })
        .for_each(|(i, x, y)| println!("{:4}: {:+.3?}---{:+.3?}", i, x, y));

    MatFile::save("VoiceCoilsMotion.mat")?
        .var("VoiceCoilsMotion", (*plant_logging.lock().await).as_slice())?;

    Ok(())
}
