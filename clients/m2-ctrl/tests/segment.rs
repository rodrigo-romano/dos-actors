use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Logging, Signals};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    FluidDampingForces, ModalCommand, VoiceCoilsForces, VoiceCoilsMotion,
};
use gmt_dos_clients_m2_ctrl::{Calibration, DataSource, Segment, SegmentCalibration};
use gmt_fem::FEM;
use matio_rs::MatFile;
use nalgebra as na;

const SID: u8 = 2;

#[tokio::test]
async fn segment() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 8000;
    let sim_duration = 1_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mut fem = FEM::from_env()?;
    // whole_fem.keep_input::<>()
    // println!("{fem}");

    let n_mode = 66;
    let n_actuator = 675;
    /*     let calibration = SegmentCalibration::new(
        SID,
        n_mode,
        n_actuator,
        (
            "calib_dt/m2asm_ctrl_dt.mat".to_string(),
            format!("V_S{SID}"),
        ),
        DataSource::Fem,
        Some(&mut fem),
    )?; */

    let mut asms_calibration = if let Ok(data) = Calibration::load("asms_stiffness.bin") {
        data
    } else {
        Calibration::new(
            n_mode,
            n_actuator,
            (
                "calib_dt/m2asm_ctrl_dt.mat".to_string(),
                (1..=7).map(|i| format!("V_S{i}")).collect::<Vec<String>>(),
            ),
            &mut fem,
        )?;
        asms_calibration.save("asms_stiffness.bin")?
    };

    let calibration = asms_calibration.remove(SID as usize - 1);

    let fem_as_state_space = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        .truncate_hankel_singular_values(4.855e-5)
        .hankel_frequency_lower_bound(50.)
        .including_m2(
            vec![calibration.modes().as_view()],
            vec![calibration.modes().transpose().as_view()],
            Some(vec![SID]),
        )?
        .build()?;
    println!("{fem_as_state_space}");
    let mut plant: Actor<_> = (fem_as_state_space, "Plant").into();

    let mut asm_setpoint: Initiator<_> = (
        Signals::new(n_mode, n_step)
            .channel(0, gmt_dos_clients::Signal::Constant(1e-6))
            .channel(n_mode - 1, gmt_dos_clients::Signal::Constant(1e-6)),
        "ASM
    Set-Point",
    )
        .into();

    let mut asm =
        Segment::<SID>::builder(n_mode, calibration, &mut asm_setpoint).build(&mut plant)?;

    let plant_logging = Logging::<f64>::new(1).into_arcx();
    let mut plant_logger: Terminator<_> = Actor::new(plant_logging.clone());

    plant
        .add_output()
        .bootstrap()
        .build::<VoiceCoilsMotion<SID>>()
        .into_input(&mut plant_logger)?;

    model!(asm_setpoint, asm, plant, plant_logger)
        .name("ASM_segment")
        .flowchart()
        .check()?
        .run()
        .await?;

    println!("{}", *plant_logging.lock().await);
    (*plant_logging.lock().await)
        .chunks()
        .enumerate()
        .skip(n_step - 21)
        .map(|(i, x)| {
            (
                i,
                x.iter().take(5).map(|x| x * 1e6).collect::<Vec<f64>>(),
                x.iter()
                    .skip(n_mode - 5)
                    .map(|x| x * 1e6)
                    .collect::<Vec<f64>>(),
            )
        })
        .for_each(|(i, x, y)| println!("{:4}: {:+.3?}---{:+.3?}", i, x, y));

    MatFile::save("VoiceCoilsMotion.mat")?
        .var("VoiceCoilsMotion", (*plant_logging.lock().await).as_slice())?;

    Ok(())
}
