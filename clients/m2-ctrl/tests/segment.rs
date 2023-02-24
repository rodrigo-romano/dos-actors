use gmt_dos_actors::prelude::*;
use gmt_dos_clients::Signals;
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    FluidDampingForces, ModalCommand, VoiceCoilsForces, VoiceCoilsMotion,
};
use gmt_dos_clients_m2_ctrl::AsmSegmentInnerController;
use gmt_fem::{
    fem_io::{MCM2S1FluidDampingF, MCM2S1VCDeltaD, MCM2S1VCDeltaF},
    FEM,
};

const SID: u8 = 1;

#[tokio::test]
async fn segment() -> anyhow::Result<()> {
    let sim_sampling_frequency = 8000;
    let sim_duration = 3_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let fem = FEM::from_env()?;
    // println!("{fem}");

    let fem_as_state_space = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        .truncate_hankel_singular_values(4.855e-5)
        .hankel_frequency_lower_bound(50.)
        .ins::<MCM2S1VCDeltaF>()
        .ins::<MCM2S1FluidDampingF>()
        .outs::<MCM2S1VCDeltaD>()
        .build()?;
    println!("{fem_as_state_space}");
    let mut plant: Actor<_> = (fem_as_state_space, "Plant").into();

    let n_mode = 66;
    let mut asm_setpoint: Initiator<_> = (
        Signals::new(n_mode, n_step),
        "ASM
    Set-Point",
    )
        .into();
    let mut asm: Actor<_> = (
        AsmSegmentInnerController::<1>::new(n_mode, None),
        format!(
            "ASM
     Segment #{SID}"
        ),
    )
        .into();

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
        .bootstrap()
        .build::<VoiceCoilsMotion<SID>>()
        .into_input(&mut asm)?;

    model!(asm_setpoint, asm, plant)
        .name("ASM_segment")
        .flowchart()
        .check()?;

    Ok(())
}
