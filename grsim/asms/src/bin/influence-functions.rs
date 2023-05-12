use std::{env, path::Path};

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Signal, Signals};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    FaceSheetFigure, VoiceCoilsForces, VoiceCoilsMotion,
};
use gmt_dos_clients_m2_ctrl::{Calibration, Segment};
use gmt_fem::{fem_io::*, FEM};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 8000;
    let n_actuator = 675;
    let n_step = 100;
    let n_mode = env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());
    let actuator_motion = 10e-6;

    let mut fem = FEM::from_env()?;

    let sids = vec![1]; //, 2, 3, 4, 5, 6, 7];
    let calibration_file_name =
        Path::new(env!("FEM_REPO")).join(format!("asms_zonal_{n_mode}kl_calibration.bin"));
    let asms_calibration = if let Ok(data) = Calibration::load(&calibration_file_name) {
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
        .outs::<M2Segment1AxialD>()
        .use_static_gain_compensation()
        .build()?;

    let mut plant: Actor<_> = (fem_as_state_space, "ASM").into();

    let mut actuators: Initiator<Signals, 1> = Signals::new(n_actuator, n_step)
        .channel(629, Signal::Constant(actuator_motion))
        .into();

    let mut asm = Segment::<1>::builder(n_actuator, asms_calibration.stiffness(1), &mut actuators)
        .build(&mut plant)?;

    let mut plant_logger: Terminator<_> = Arrow::builder(n_step).build().into();

    asm.add_output()
        .build::<VoiceCoilsForces<1>>()
        .logn(&mut plant_logger, n_actuator)
        .await?;
    plant
        .add_output()
        .build::<VoiceCoilsMotion<1>>()
        .logn(&mut plant_logger, n_actuator)
        .await?;
    plant
        .add_output()
        .build::<FaceSheetFigure<1>>()
        .logn(&mut plant_logger, n_actuator)
        .await?;

    (asm + actuators + plant + plant_logger)
        .name("ASM_influence-function")
        .flowchart()
        .check()?
        .run()
        .await?;

    Ok(())
}
