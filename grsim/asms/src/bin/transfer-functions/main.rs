use std::{env, path::Path};

use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{interface::UID, Signal, Signals};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    AsmCommand, FaceSheetFigure, VoiceCoilsForces, VoiceCoilsMotion,
};
use gmt_dos_clients_m2_ctrl::{Calibration, Segment};
use gmt_fem::{
    fem_io::{
        actors_inputs::{MCM2S1FluidDampingF, MCM2S1VCDeltaF},
        actors_outputs::{M2Segment1AxialD, MCM2S1VCDeltaD},
    },
    FEM,
};

use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update, Write};
use nalgebra as na;

/// Gain
pub struct Gain {
    u: na::DVector<f64>,
    y: na::DVector<f64>,
    mat: na::DMatrix<f64>,
}
impl Gain {
    pub fn new(mat: na::DMatrix<f64>) -> Self {
        Self {
            u: na::DVector::zeros(mat.ncols()),
            y: na::DVector::zeros(mat.nrows()),
            mat,
        }
    }
}
impl Update for Gain {
    fn update(&mut self) {
        self.y = &self.mat * &self.u;
    }
}
impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for Gain {
    fn read(&mut self, data: Data<U>) {
        self.u = na::DVector::from_row_slice(&data);
    }
}
impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for Gain {
    fn write(&mut self) -> Option<Data<U>> {
        Some(Data::new(self.y.as_slice().to_vec()))
    }
}

pub struct Select {
    index: usize,
    data: f64,
}
impl Select {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            data: Default::default(),
        }
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
        self.data = *(data.get(self.index).unwrap());
    }
}
impl<T: UniqueIdentifier<DataType = Vec<f64>>> Write<T> for Select {
    fn write(&mut self) -> Option<Data<T>> {
        Some(Data::new(vec![self.data]))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 8000;
    let n_actuator = 675;
    let n_step = 8000;
    let n_mode = env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());

    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("bin")
        .join("transfer-functions");
    env::set_var("DATA_REPO", repo);

    let mut fem = Option::<FEM>::None;

    let sids = vec![1]; //, 2, 3, 4, 5, 6, 7];
    let calibration_file_name = format!("asms_zonal_{n_mode}kl_calibration.bin");
    let mut asms_calibration = Calibration::try_from(calibration_file_name.as_str())
        .unwrap_or_else(|_| {
            let asms_calibration = Calibration::builder(
                n_mode,
                n_actuator,
                (
                    "KLmodesQR.mat".to_string(),
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

    let fem_as_state_space = DiscreteModalSolver::<ExponentialMatrix>::try_from(
        "fem_state-space.bin",
    )
    .unwrap_or_else(|_| {
        let dss = DiscreteModalSolver::<ExponentialMatrix>::from_fem(
            fem.unwrap_or(FEM::from_env().expect("failed to load the FEM from `FEM_REPO`")),
        )
        .sampling(sim_sampling_frequency as f64)
        .proportional_damping(2. / 100.)
        .ins::<MCM2S1VCDeltaF>()
        .ins::<MCM2S1FluidDampingF>()
        .outs::<MCM2S1VCDeltaD>()
        .outs_with::<M2Segment1AxialD>(asms_calibration.modes_t(Some(sids.clone())).unwrap()[0])
        .use_static_gain_compensation()
        .build()
        .expect("failed to build the FEM state space solver");
        dss.save("fem_state-space.bin")
            .expect("failed to save the FEM state space solver");
        dss
    });
    println!("{fem_as_state_space}");
    let mut plant: Actor<_> = (fem_as_state_space, "ASM FEM").into();

    let mode = env::args().nth(1).map_or_else(|| 1, |x| x.parse().unwrap());
    let mut signals = Signals::new(n_mode, n_step).channel(
        dbg!(mode) - 1,
        Signal::white_noise()
            .expect("fishy!")
            .std_dev(1e-7)
            .expect("very fishy!"),
    );
    signals.progress();
    let mut asm_setpoint: Initiator<Signals, 1> = (signals, "White Noise").into();
    let mut select_u: Actor<_> = Select::new(mode - 1).into();
    let mut select_y: Actor<_> = Select::new(mode - 1).into();

    let mut modes2forces: Actor<_> = (
        Gain::new(asms_calibration.modes(Some(sids))[0].into()),
        "Modes 2 Forces",
    )
        .into();

    let asm = Segment::<1>::builder(n_actuator, asms_calibration.stiffness(1), &mut modes2forces)
        .build(&mut plant)?;

    let mut plant_logger: Terminator<_> = Arrow::builder(n_step).build().into();

    asm_setpoint
        .add_output()
        .multiplex(2)
        .build::<AsmCommand<1>>()
        .into_input(&mut modes2forces)
        .into_input(&mut select_u)?;
    select_u
        .add_output()
        .build::<U>()
        .logn(&mut plant_logger, 1)
        .await?;
    plant
        .add_output()
        .build::<FaceSheetFigure<1>>()
        .into_input(&mut select_y)?;
    select_y
        .add_output()
        .build::<Y>()
        .logn(&mut plant_logger, 1)
        .await?;

    (asm + asm_setpoint + modes2forces + plant + plant_logger + select_u + select_y)
        .name("ASM_transfer-function")
        .flowchart()
        .check()?
        .run()
        .await?;

    Ok(())
}
