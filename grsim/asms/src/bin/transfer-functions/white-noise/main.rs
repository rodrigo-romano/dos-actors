use std::{env, path::Path};

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{interface::UID, Gain, Signal, Signals};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    AsmCommand, FaceSheetFigure, VoiceCoilsForces, VoiceCoilsMotion,
};
use gmt_dos_clients_m2_ctrl::{Calibration, Segment};
use gmt_fem::FEM;

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update, Write};

pub struct Select {
    index: Vec<usize>,
    data: Vec<f64>,
}
impl Select {
    pub fn new(index: Vec<usize>) -> Self {
        let data = vec![0f64; index.len()];
        Self { index, data }
    }
}
impl Update for Select {
    fn update(&mut self) {}
}
#[derive(UID)]
enum U {}
#[derive(UID)]
enum Y {}
impl<T: UniqueIdentifier<DataType = Vec<f64>>> Read<T> for Select {
    fn read(&mut self, data: Data<T>) {
        self.index
            .iter()
            .zip(self.data.iter_mut())
            .for_each(|(i, v)| {
                *v = *(data.get(*i).unwrap());
            });
    }
}
impl<T: UniqueIdentifier<DataType = Vec<f64>>> Write<T> for Select {
    fn write(&mut self) -> Option<Data<T>> {
        Some(Data::new(self.data.clone()))
    }
}

const SID: u8 = 1;
type VCDeltaF = gmt_dos_clients_fem::fem_io::actors_inputs::MCM2S1VCDeltaF;
type FluidDampingF = gmt_dos_clients_fem::fem_io::actors_inputs::MCM2S1FluidDampingF;
type VCDeltaD = gmt_dos_clients_fem::fem_io::actors_outputs::MCM2S1VCDeltaD;
type AxialD = gmt_dos_clients_fem::fem_io::actors_outputs::M2Segment1AxialD;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    env!("FEM_REPO");

    let sim_sampling_frequency = 8000;
    let n_actuator = 675;
    let n_step = 1000 + 8000 * 10;
    let n_mode = 496; //env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());

    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("bin")
        .join("transfer-functions");
    env::set_var("DATA_REPO", &repo);

    let mut fem = Option::<FEM>::None;

    let sids = vec![SID]; //, 2, 3, 4, 5, 6, 7];
    let calibration_file_name = format!("asms_zonal_kl{n_mode}gs36_calibration.bin");
    let mut asms_calibration = Calibration::try_from(calibration_file_name.as_str())
        .unwrap_or_else(|_| {
            let asms_calibration = Calibration::builder(
                n_mode,
                n_actuator,
                (
                    repo.join("KLmodesGS36.mat").to_str().unwrap().into(),
                    (1..=7).map(|i| format!("KL_{i}")).collect::<Vec<String>>(),
                ),
                fem.get_or_insert(FEM::from_env().expect("failed to load the FEM from `FEM_REPO`")),
            )
            .stiffness("Zonal")
            .build()
            .expect("failed to calibrate the ASMS");
            asms_calibration
                .save(&calibration_file_name)
                .expect("failed to save ASMS calibration");
            asms_calibration
        });
    asms_calibration.transpose_modes();

    let file_name = format!("fem_state-space_kl{n_mode}gs36.bin");
    let fem_as_state_space = DiscreteModalSolver::<ExponentialMatrix>::try_from(file_name.as_str())
        .unwrap_or_else(|_| {
            let dss = DiscreteModalSolver::<ExponentialMatrix>::from_fem(
                fem.unwrap_or(FEM::from_env().expect("failed to load the FEM from `FEM_REPO`")),
            )
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .ins::<VCDeltaF>()
            .ins::<FluidDampingF>()
            .outs::<VCDeltaD>()
            .outs_with::<AxialD>(asms_calibration.modes_t(Some(sids.clone())).unwrap()[0])
            .use_static_gain_compensation()
            // .truncate_hankel_singular_values(1e-3)
            .build()
            .expect("failed to build the FEM state space solver");
            dss.save(file_name)
                .expect("failed to save the FEM state space solver");
            dss
        });
    println!("{fem_as_state_space}");
    let mut plant: Actor<_> = (fem_as_state_space, "ASM FEM").into();

    let last_mode = env::args().nth(1).map_or_else(|| 1, |x| x.parse().unwrap());
    let mut mode: Vec<usize> = (0..=dbg!(last_mode) - 1).collect();
    mode.dedup();
    let mut signals = mode.iter().fold(Signals::new(n_mode, n_step), |s, i| {
        s.channel(
            *i,
            Signal::white_noise()
                .expect("fishy!")
                .std_dev(1e-7)
                .expect("very fishy!"),
        )
    });
    signals.progress();
    let mut asm_setpoint: Initiator<Signals, 1> = (signals, "White Noise").into();
    let mut select_u: Actor<_> = (
        Select::new(mode.clone()),
        "Select 
mode(s)",
    )
        .into();
    let mut select_y: Actor<_> = (
        Select::new(mode.clone()),
        "Select 
mode(s)",
    )
        .into();

    let gain: nalgebra::DMatrix<f64> = asms_calibration.modes(Some(sids))[0].into();
    let mut modes2forces: Actor<_> = (Gain::new(gain), "Modes 2 Positions").into();

    let asm = Segment::<SID>::builder(
        n_actuator,
        asms_calibration.stiffness(SID),
        &mut modes2forces,
    )
    .build(&mut plant)?;

    let mut plant_logger: Terminator<_> = Arrow::builder(n_step).build().into();

    asm_setpoint
        .add_output()
        .multiplex(2)
        .build::<AsmCommand<SID>>()
        .into_input(&mut modes2forces)
        .into_input(&mut select_u)?;
    select_u
        .add_output()
        .build::<U>()
        .logn(&mut plant_logger, mode.len())
        .await?;
    plant
        .add_output()
        .build::<FaceSheetFigure<SID>>()
        .into_input(&mut select_y)?;
    select_y
        .add_output()
        .build::<Y>()
        .logn(&mut plant_logger, mode.len())
        .await?;

    (asm + asm_setpoint + modes2forces + plant + plant_logger + select_u + select_y)
        .name("ASM_transfer-function")
        .flowchart()
        .check()?
        .run()
        .await?;

    Ok(())
}
