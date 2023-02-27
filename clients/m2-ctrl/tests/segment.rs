use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{Logging, Signals};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    FluidDampingForces, ModalCommand, VoiceCoilsForces, VoiceCoilsMotion,
};
use gmt_dos_clients_m2_ctrl::AsmSegmentInnerController;
use gmt_fem::{
    fem_io::{self, MCM2S1FluidDampingF, MCM2S1VCDeltaD, MCM2S1VCDeltaF},
    FEM,
};
use matio_rs::MatFile;
use nalgebra as na;

const SID: u8 = 1;

#[tokio::test]
async fn segment() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 8000;
    let sim_duration = 1_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let whole_fem = FEM::from_env()?;
    // whole_fem.keep_input::<>()
    // println!("{fem}");

    /*     log::info!("Computing the gain: MCM2S1VCDeltaF->MCM2S1VCDeltaD");
    let vc_f2d = {
        let mut fem = whole_fem.clone();
        fem.keep_input::<fem_io::MCM2S1VCDeltaF>()
            .and_then(|fem| fem.keep_output::<fem_io::MCM2S1VCDeltaD>())
            .map(|fem| fem.static_gain())
            .expect("failed to compute the gain MCM2S1VCDeltaF->MCM2S1VCDeltaD")
    };
    dbg!(vc_f2d.shape()); */

    log::info!(r#"Loading Ks and V from "m2asm_ctrl_dt.mat""#);
    let n_mode = 66;
    let mat = MatFile::load("calib_dt/m2asm_ctrl_dt.mat")?;
    /*     let ks_s1: Vec<f64> = mat.var("KsS1_66")?;
    let ks_s1: Vec<f64> = (0..n_mode)
        .flat_map(|i| {
            ks_s1
                .iter()
                .skip(i)
                .step_by(n_mode)
                .cloned()
                .collect::<Vec<f64>>()
        })
        .collect(); */
    let v_s1: Vec<f64> = mat.var("V_S1")?;
    // dbg!(v_s1.len());
    let v_s1: na::DMatrix<f64> = na::DMatrix::from_column_slice(675, n_mode, &v_s1);
    // dbg!(v_s1.shape());

    /*     log::info!("Computing the modal stiffness");
    let stiffness_mat = (v_s1.transpose() * vc_f2d * &v_s1)
        .try_inverse()
        .expect("failed to compute stiffness matrix");
    let stiffness: Vec<f64> = stiffness_mat
        .row_iter()
        .flat_map(|row| row.iter().cloned().collect::<Vec<f64>>())
        .collect();
    MatFile::save("stiffness.mat")?.var("stiffness", stiffness)?; */

    let mat: Vec<f64> = MatFile::load("stiffness.mat")?.var("stiffness")?;
    let stiffness_mat = na::DMatrix::<f64>::from_column_slice(n_mode, n_mode, &mat);
    let stiffness: Vec<f64> = stiffness_mat
        .row_iter()
        .flat_map(|row| row.iter().cloned().collect::<Vec<f64>>())
        .collect();

    let fem_as_state_space = DiscreteModalSolver::<ExponentialMatrix>::from_fem(whole_fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        .truncate_hankel_singular_values(4.855e-5)
        .hankel_frequency_lower_bound(50.)
        .ins_with::<MCM2S1VCDeltaF>(v_s1.as_view())
        .ins_with::<MCM2S1FluidDampingF>(v_s1.as_view())
        .outs_with::<MCM2S1VCDeltaD>(v_s1.transpose().as_view())
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

    let mut asm: Actor<_> = (
        AsmSegmentInnerController::<1>::new(n_mode, Some(stiffness)),
        format!(
            "ASM
     Segment #{SID}"
        ),
    )
        .into();

    let plant_logging = Logging::<f64>::new(1).into_arcx();
    let mut plant_logger: Terminator<_> = Actor::new(plant_logging.clone());

    asm_setpoint
        .add_output()
        .build::<ModalCommand<SID>>()
        .into_input(&mut asm)?;
    asm.add_output()
        .build::<VoiceCoilsForces<SID>>()
        .into_input(&mut plant)?;
    asm.add_output()
        .build::<FluidDampingForces<SID>>()
        .into_input(&mut plant)?;
    plant
        .add_output()
        .multiplex(2)
        .bootstrap()
        .build::<VoiceCoilsMotion<SID>>()
        .into_input(&mut asm)
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
    Ok(())
}
