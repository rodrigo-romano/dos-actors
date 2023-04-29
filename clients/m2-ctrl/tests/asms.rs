use std::{env, path::Path};

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Logging, Signal, Signals};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::VoiceCoilsMotion;
use gmt_dos_clients_m2_ctrl::{Calibration, Segment};
use gmt_fem::FEM;
use matio_rs::MatFile;
use nalgebra::DVector;
use nanorand::{Rng, WyRand};

#[tokio::test]
async fn asms() -> anyhow::Result<()> {
    env_logger::init();

    let mut rng = WyRand::new();

    let sim_sampling_frequency = 8000;
    let sim_duration = 1_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mut fem = FEM::from_env()?;
    // whole_fem.keep_input::<>()
    // println!("{fem}");

    let n_mode = env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());
    let n_actuator = 675;

    let sids = vec![1, 3, 4, 5, 6, 7];
    let calibration_file_name = Path::new(env!("FEM_REPO")).join("none"); //.join(format!("asms_{n_mode}kl_calibration.bin"));
    let mut asms_calibration = if let Ok(data) = Calibration::load(&calibration_file_name) {
        data
    } else {
        let asms_calibration = Calibration::builder(
            n_mode,
            n_actuator,
            (
                "KLmodes.mat".to_string(),
                (1..=7).map(|i| format!("KL_{i}")).collect::<Vec<String>>(),
            ),
            &mut fem,
        )
        .stiffness("Zonal")
        .build()?;
        asms_calibration.save(&calibration_file_name)?;
        Calibration::load(calibration_file_name)?
    };
    asms_calibration.transpose_modes();

    let fem_as_state_space = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        // .truncate_hankel_singular_values(1.531e-3)
        // .hankel_frequency_lower_bound(50.)
/*         .including_asms(Some(sids.clone()),
        Some(asms_calibration.modes(Some(sids.clone()))),
         Some(asms_calibration.modes_t(Some(sids.clone())))
        .expect(r#"expect some transposed modes, found none (have you called "Calibration::transpose_modes"#))? */
        .including_asms(Some(sids.clone()),
        None,
        None)?
        .use_static_gain_compensation()
        .build()?;
    println!("{fem_as_state_space}");
    let mut plant: Actor<_> = (fem_as_state_space, "Plant").into();

    let plant_logging = Logging::<f64>::new(sids.len()).into_arcx();
    let mut plant_logger: Terminator<_> = Actor::new(plant_logging.clone());

    let mut m2: Model<model::Unknown> = Default::default();
    let mut setpoints: Model<model::Unknown> = Default::default();

    let mut aas = vec![];
    let mut asms_coefs = |sid: u8| {
        let mut c = 0;
        let mut n = 0;
        let mut a = vec![];
        for i in 0..n_mode {
            if c < (n + 1) {
                c += 1;
            } else {
                n += 1;
                c = 1;
            }
            a.push(1e-6 * (rng.generate::<f64>() * 2f64 - 1f64) / ((n + 1) as f64));
            // asm_coefs = asm_coefs.channel(i, gmt_dos_clients::Signal::Constant(a));
        }
        let m = asms_calibration.modes(Some(vec![dbg!(sid)]))[0];
        let mt = asms_calibration.modes_t(Some(vec![dbg!(sid)])).unwrap()[0];
        dbg!(m.shape());
        dbg!((a[0], a[n_mode - 1]));
        let u = DVector::from_column_slice(&a);
        dbg!(u.shape());
        let forces = m * u;
        let a_u = mt * &forces;
        dbg!((a_u[0], a_u[n_mode - 1]));
        aas.extend_from_slice(a_u.as_slice());
        forces
            .into_iter()
            .enumerate()
            .fold(Signals::new(n_actuator, n_step), |s, (i, f)| {
                s.channel(i, Signal::Constant(*f))
            })
    };

    for &sid in &sids {
        match sid {
            i if i == 1 => {
                let mut asm_setpoint: Initiator<_> = (
                    asms_coefs(i),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                let client = asm_setpoint.client();
                (&mut *client.lock().await).progress();
                m2 += Segment::<1>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asm_setpoint,
                )
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
                    asms_coefs(i),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 += Segment::<2>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asm_setpoint,
                )
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
                    asms_coefs(i),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 += Segment::<3>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asm_setpoint,
                )
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
                    asms_coefs(i),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 += Segment::<4>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asm_setpoint,
                )
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
                    asms_coefs(i),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 += Segment::<5>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asm_setpoint,
                )
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
                    asms_coefs(i),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 += Segment::<6>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asm_setpoint,
                )
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
                    asms_coefs(i),
                    format!(
                        "ASM #{i}
      Set-Point"
                    ),
                )
                    .into();
                m2 += Segment::<7>::builder(
                    n_actuator,
                    asms_calibration.stiffness(i),
                    &mut asm_setpoint,
                )
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

    println!("{}", *plant_logging.lock().await);

    let n = sids.len();
    aas.chunks(n_mode)
        .enumerate()
        // .skip(n_step - 21)
        .map(|(i, x)| {
            (
                i,
                x.iter()
                    .step_by(n_mode)
                    .take(n)
                    .map(|x| x * 1e6)
                    .collect::<Vec<f64>>(),
                x.iter()
                    .skip(n_mode - 1)
                    .step_by(n_mode)
                    .take(n)
                    .map(|x| x * 1e6)
                    .collect::<Vec<f64>>(),
            )
        })
        .for_each(|(i, x, y)| println!("{:4}: {:+.3?}---{:+.3?}", i, x, y));

    let m = asms_calibration.modes_t(Some(sids.clone())).unwrap();

    (*plant_logging.lock().await)
        .chunks()
        .enumerate()
        .skip(n_step - 21)
        .map(|(i, u)| {
            let x: Vec<_> = u
                .chunks(n_actuator)
                .zip(&m)
                .flat_map(|(u, m)| {
                    let c = m * DVector::<f64>::from_column_slice(u);
                    c.as_slice().to_vec()
                })
                .collect();
            (
                i,
                x.iter()
                    .step_by(n_mode)
                    .take(n)
                    .map(|x| x * 1e6)
                    .collect::<Vec<f64>>(),
                x.iter()
                    .skip(n_mode - 1)
                    .step_by(n_mode)
                    .take(n)
                    .map(|x| x * 1e6)
                    .collect::<Vec<f64>>(),
            )
        })
        .for_each(|(i, x, y)| println!("{:4}: {:+.3?}---{:+.3?}", i, x, y));

    /*     MatFile::save("VoiceCoilsMotion.mat")?
    .var("VoiceCoilsMotion", (*plant_logging.lock().await).as_slice())?; */

    let aas_err = (*plant_logging.lock().await)
        .chunks()
        .last()
        .unwrap()
        .chunks(n_mode)
        .zip(aas.chunks(n_mode))
        .map(|(x, x0)| {
            x.iter()
                .zip(x0)
                .map(|(x, x0)| (x - x0) * 1e6)
                .map(|x| x * x)
                .sum::<f64>()
                / n_mode as f64
        })
        .map(|x| x.sqrt())
        .sum::<f64>()
        / 7f64;

    assert!(dbg!(aas_err) < 1e-4);

    Ok(())
}
