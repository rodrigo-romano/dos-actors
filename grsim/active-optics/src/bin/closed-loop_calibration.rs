use std::{collections::VecDeque, env, mem, path::Path, time::Instant};

use crseo::{
    wavefrontsensor::{
        DifferentialPistonSensor, GeomShack, SegmentCalibration, Slopes, SlopesArray, DOF, RBM,
    },
    Builder, FromBuilder, Gmt, SegmentWiseSensorBuilder, Source, WavefrontSensorBuilder,
};
use gmt_dos_actors::prelude::*;
use gmt_dos_clients::{
    interface::{Data, Read, UniqueIdentifier, Update, Write},
    Integrator, Logging, Sampler,
};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_crseo::{M2modes, SegmentWfeRms};
use gmt_dos_clients_io::gmt_m1;
use matio_rs::MatFile;
use ngao::{GuideStar, LittleOpticalModel, ResidualM2modes, SensorData, WavefrontSensor};
use skyangle::Conversion;

const AGWS: usize = 20;
const PUSHPULL: usize = AGWS * 1;

#[derive(Default, Clone, Debug)]
pub enum CalibrationCommand {
    #[default]
    Push,
    Reset(Box<CalibrationCommand>),
    Pull,
}

pub struct CalibrationSignals {
    command_kind: CalibrationCommand,
    dof: VecDeque<usize>,
    rbm: RBM,
    sid: u8,
    stroke: f64,
    command: Option<Vec<f64>>,
    index: usize,
}
#[derive(Default)]
pub struct CalibrationMatrix {
    command_kind: CalibrationCommand,
    stroke: f64,
    matrix: Vec<Slopes>,
    data: Slopes,
}
impl From<&CalibrationSignals> for CalibrationMatrix {
    fn from(value: &CalibrationSignals) -> Self {
        Self {
            stroke: value.stroke,
            ..Default::default()
        }
    }
}
impl CalibrationSignals {
    pub fn new<R: Into<RBM>>(rbm: R) -> Self {
        let rbm = rbm.into();
        let dof = CalibrationSignals::dof_iter(&rbm);
        Self {
            command_kind: CalibrationCommand::Push,
            dof,
            rbm,
            sid: 1,
            stroke: 1e-6,
            command: Some(vec![0f64; 6]),
            index: 0,
        }
    }
    fn dof_iter(rbm: &RBM) -> VecDeque<usize> {
        match rbm {
            RBM::Txyz(dof) => dof
                .clone()
                .unwrap_or(DOF::Range(0..3))
                .into_iter()
                .collect::<VecDeque<usize>>(),
            RBM::Rxyz(dof) => dof
                .clone()
                .unwrap_or(DOF::Range(0..3))
                .into_iter()
                .map(|i| i + 3)
                .collect::<VecDeque<usize>>(),
            RBM::TRxyz => (0..6).collect::<VecDeque<usize>>(),
        }
    }
}

impl From<&mut CalibrationMatrix> for SlopesArray {
    fn from(value: &mut CalibrationMatrix) -> Self {
        mem::take(&mut value.matrix).into()
    }
}

impl Update for CalibrationSignals {
    fn update(&mut self) {
        match &self.command_kind {
            CalibrationCommand::Push => {
                self.index = if let Some(index) = self.dof.pop_front() {
                    index
                } else {
                    if self.sid == 7 {
                        self.command = None;
                        return;
                    } else {
                        self.sid += 1;
                        self.dof = CalibrationSignals::dof_iter(&self.rbm);
                        self.dof.pop_front().unwrap()
                    }
                };
                log::info!("Push dof#{} of segment #{}", self.index + 1, self.sid);
                self.command.as_mut().map(|c| {
                    c.fill(0f64);
                    c[self.index] = self.stroke;
                });
                self.command_kind = CalibrationCommand::Reset(Box::new(CalibrationCommand::Push));
            }
            CalibrationCommand::Reset(kind) => {
                log::info!("Reset");
                self.command.as_mut().map(|c| c.fill(0f64));
                self.command_kind = match **kind {
                    CalibrationCommand::Push => CalibrationCommand::Pull,
                    CalibrationCommand::Reset(_) => todo!(),
                    CalibrationCommand::Pull => CalibrationCommand::Push,
                }
            }
            CalibrationCommand::Pull => {
                log::info!("Pull");
                self.command.as_mut().map(|c| c[self.index] = -self.stroke);
                self.command_kind = CalibrationCommand::Reset(Box::new(CalibrationCommand::Pull));
            }
        }
    }
}

impl<const ID: u8> Write<gmt_m1::segment::RBM<ID>> for CalibrationSignals {
    fn write(&mut self) -> Option<Data<gmt_m1::segment::RBM<ID>>> {
        self.command.as_ref().map(|c| {
            if self.sid == ID {
                c.to_vec().into()
            } else {
                vec![0f64; 6].into()
            }
        })
    }
}

pub enum PushPull {}
impl UniqueIdentifier for PushPull {
    type DataType = CalibrationCommand;
}
impl Write<PushPull> for CalibrationSignals {
    fn write(&mut self) -> Option<Data<PushPull>> {
        Some(Data::new(self.command_kind.clone()))
    }
}
impl Update for CalibrationMatrix {
    fn update(&mut self) {
        if let CalibrationCommand::Reset(value) = &self.command_kind {
            match **value {
                CalibrationCommand::Push => self.matrix.push(mem::take(&mut self.data)),
                CalibrationCommand::Reset(_) => (),
                CalibrationCommand::Pull => {
                    self.matrix.last_mut().map(|mut push| {
                        push -= mem::take(&mut self.data);
                        push *= 0.5 * self.stroke.recip() as f32;
                    });
                }
            }
        }
    }
}
impl Read<PushPull> for CalibrationMatrix {
    fn read(&mut self, data: Data<PushPull>) {
        self.command_kind = (*data).clone();
    }
}
impl Read<SensorData> for CalibrationMatrix {
    fn read(&mut self, data: Data<SensorData>) {
        self.data = (*data).clone().into();
    }
}

fn setup_calibratons<const K: u8>(
    calibrations_signal: &mut Initiator<CalibrationSignals, AGWS>,
    gom_act: &mut Actor<LittleOpticalModel, 1, 1>,
    agws_act: &mut Actor<LittleOpticalModel, AGWS, AGWS>,
    // cal_logger: &mut Terminator<Logging<f64>, AGWS>,
) -> anyhow::Result<
    Actor<Sampler<Vec<f64>, gmt_m1::segment::RBM<K>, gmt_m1::segment::RBM<K>>, AGWS, 1>,
> {
    let mut upsampler: Actor<_, AGWS, 1> = (
        Sampler::default(),
        format!(
            "M1S{K} 
{AGWS}:1"
        ),
    )
        .into();
    calibrations_signal
        .add_output()
        .multiplex(2)
        .build::<gmt_m1::segment::RBM<K>>()
        .into_input(&mut upsampler)
        .into_input(agws_act)?;
    // .into_input(cal_logger)?;
    upsampler
        .add_output()
        .build::<gmt_m1::segment::RBM<K>>()
        .into_input(gom_act)?;
    Ok(upsampler)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let data_repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    dbg!(&data_repo);
    env::set_var("DATA_REPO", &data_repo);
    env::set_var("GMT_MODES_PATH", &data_repo);

    let sampling_frequency = 1usize; // Hz
                                     // let sim_duration = 1usize;
                                     // assert_eq!(sampling_frequency / PYWFS_READOUT, 4000);
                                     // assert_eq!(sampling_frequency / PYWFS, 4000);

    let n_lenslet = 92;
    let n_mode: usize = env::var("N_KL_MODE").map_or_else(|_| 66, |x| x.parse::<usize>().unwrap());

    let builder = GeomShack::builder().lenslet(n_lenslet, 8);
    let src_builder = builder.guide_stars(None);

    let m2_modes = "M2_OrthoNorm_KarhunenLoeveModes";
    // let m2_modes = "Karhunen-Loeve";

    let now = Instant::now();
    let mut slopes_mat = builder.clone().calibrate(
        SegmentCalibration::modes(m2_modes, 1..n_mode, "M2"),
        src_builder.clone(),
    );
    println!(
        "M2 {}modes/segment calibrated in {}s",
        n_mode,
        now.elapsed().as_secs()
    );
    slopes_mat.pseudo_inverse(None).unwrap();

    let gmt_builder = Gmt::builder().m2(m2_modes, n_mode);
    let gom = LittleOpticalModel::builder()
        .gmt(gmt_builder.clone())
        .source(src_builder)
        .sampling_frequency(sampling_frequency as f64)
        .build()?
        .into_arcx();

    let mut gom_act: Actor<_> = Actor::new(gom.clone()).name("GS>>(GMT+ASMS)");

    let mut sensor: Actor<_> = (
        WavefrontSensor::new(builder.build()?, slopes_mat),
        "Geometric
Shack-Hartmann",
    )
        .into();

    let fov = 12f32.from_arcmin();
    let n_agws_gs = 4;
    let agws_sh48_builder = GeomShack::builder().size(n_agws_gs).lenslet(48, 8);
    let agws_gs_builder =
        agws_sh48_builder.guide_stars(Some(Source::builder().size(n_agws_gs).on_ring(fov / 2f32)));
    let dfs_builder = DifferentialPistonSensor::builder()
        .pupil_sampling(agws_sh48_builder.pupil_sampling())
        .size(n_agws_gs);
    /*     let mut agws_sh48: Actor<_, AGWS, AGWS> = (
        WavefrontSensor::new(agws_sh48_builder.build()?, Default::default()),
        "AGWS SH48x3",
    )
        .into(); */

    let mut dfs_calibration = dfs_builder.clone().calibrate(
        SegmentCalibration::rbm("TRxyz", "M1").keep_all(),
        agws_gs_builder.clone(),
    );
    dfs_calibration = dfs_calibration.flatten()?;

    let mut agws_dfs: Actor<_, AGWS, AGWS> = (
        WavefrontSensor::new(dfs_builder.build()?, dfs_calibration),
        "DFS",
    )
        .into();
    let agws = LittleOpticalModel::builder()
        .gmt(gmt_builder)
        .source(agws_gs_builder)
        .sampling_frequency(sampling_frequency as f64)
        .build()?
        .into_arcx();
    let mut agws_act: Actor<_, AGWS, AGWS> = Actor::new(agws).name(format!(
        "AGWS {n_agws_gs} GS
>> (GMT+ASMS)"
    ));

    let signal = CalibrationSignals::new("TRxyz");
    let matrix = CalibrationMatrix::from(&signal).into_arcx();
    let mut calibrations_signal = Initiator::<_, AGWS>::new(signal.into_arcx()).name(
        "Calibration
Signals",
    );
    let mut calibrations_matrix = Terminator::<_, AGWS>::new(matrix.clone()).name(
        "Calibration
Matrix",
    );

    /*     let logging = Logging::new(1).into_arcx();
    let mut logger: Terminator<_, AGWS> = Actor::new(logging.clone());

    let cal_logging = Logging::new(7).into_arcx();
    let mut cal_logger: Terminator<_, AGWS> = Actor::new(cal_logging.clone()); */

    /*     let m2_logging = Logging::new(1).into_arcx();
    let mut m2_logger: Terminator<_, PUSHPULL> = Actor::new(m2_logging.clone());

    let agws_logging = Logging::new(1).into_arcx();
    let mut agws_logger: Terminator<_, AGWS> = Actor::new(agws_logging.clone()); */
    /*     let logging = Arrow::builder(n_sample)
        .filename("ngao.parquet")
        .build()
        .into_arcx();
    let piston_logging = Logging::new(1).into_arcx();
    let mut piston_logger: Terminator<_, HDFS> = Actor::new(piston_logging.clone()).name(
        "HDFS
    Logger",
    ); */

    let mut integrator: Actor<_> = Integrator::new((n_mode - 1) * 7).gain(0.85).into();

    let mut downsampler: Actor<_, 1, PUSHPULL> = (
        Sampler::default(),
        format!(
            "ASMS
1:{AGWS}"
        ),
    )
        .into();

    let model = setup_calibratons::<1>(
        &mut calibrations_signal,
        &mut gom_act,
        &mut agws_act,
        // &mut cal_logger,
    )? + setup_calibratons::<2>(
        &mut calibrations_signal,
        &mut gom_act,
        &mut agws_act,
        // &mut &mut cal_logger,
    )? + setup_calibratons::<3>(
        &mut calibrations_signal,
        &mut gom_act,
        &mut agws_act,
        // &mut cal_logger,
    )? + setup_calibratons::<4>(
        &mut calibrations_signal,
        &mut gom_act,
        &mut agws_act,
        // &mut cal_logger,
    )? + setup_calibratons::<5>(
        &mut calibrations_signal,
        &mut gom_act,
        &mut agws_act,
        // &mut cal_logger,
    )? + setup_calibratons::<6>(
        &mut calibrations_signal,
        &mut gom_act,
        &mut agws_act,
        // &mut cal_logger,
    )? + setup_calibratons::<7>(
        &mut calibrations_signal,
        &mut gom_act,
        &mut agws_act,
        // &mut cal_logger,
    )?;

    gom_act
        .add_output()
        .build::<GuideStar>()
        .into_input(&mut sensor)?;
    sensor
        .add_output()
        .build::<ResidualM2modes>()
        .into_input(&mut integrator)?;

    integrator
        .add_output()
        .multiplex(2)
        .bootstrap()
        .build::<M2modes>()
        .into_input(&mut gom_act)
        .into_input(&mut downsampler)?;
    downsampler
        .add_output()
        // .multiplex(2)
        .build::<M2modes>()
        // .into_input(&mut m2_logger)
        .into_input(&mut agws_act)?;
    agws_act
        .add_output()
        .build::<GuideStar>()
        .into_input(&mut agws_dfs)?;
    /*     agws_act
    .add_output()
    .build::<SegmentWfeRms>()
    .into_input(&mut agws_logger)?; */
    agws_dfs
        .add_output()
        // .multiplex(2)
        .build::<SensorData>()
        // .into_input(&mut logger)
        .into_input(&mut calibrations_matrix)?;
    calibrations_signal
        .add_output()
        .build::<PushPull>()
        .into_input(&mut calibrations_matrix)?;

    let agws_model = agws_act + agws_dfs + downsampler;

    (model
        + model!(
            calibrations_signal,
            calibrations_matrix,
            gom_act,
            sensor,
            integrator
        )
        + agws_model)
        .name("closed-loop_calibration")
        .flowchart()
        .check()?
        .run()
        .await?;

    /*     println!("{}", &logging.lock().await);
    println!("{}", &cal_logging.lock().await); */
    // println!("{}", &m2_logging.lock().await);

    /*     println!("Calibration");
    (&cal_logging.lock().await)
        .chunks()
        .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
        .enumerate()
        .for_each(|(i, x)| println!("{:2}{:+.1?}", i, x));
    println!("DFS");
    (&logging.lock().await)
        .chunks()
        .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
        .enumerate()
        .for_each(|(i, x)| println!("{:2}{:+.3?}", i, x));

    println!("DFS");
    (&logging.lock().await)
        .chunks()
        .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
        .enumerate()
        .for_each(|(i, x)| println!("{:2}{:+.3?}", i, x));
    let push = (&logging.lock().await).chunks().nth(0).unwrap().to_vec();
    let pull = (&logging.lock().await).chunks().nth(2).unwrap().to_vec();
    let diff: Vec<_> = push
        .into_iter()
        .zip(pull.into_iter())
        .map(|(x, y)| 0.5 * (x - y) / 1e-6)
        .collect();
    dbg!(&diff);
    let push = (&logging.lock().await).chunks().nth(4).unwrap().to_vec();
    let pull = (&logging.lock().await).chunks().nth(6).unwrap().to_vec();
    let diff: Vec<_> = push
        .into_iter()
        .zip(pull.into_iter())
        .map(|(x, y)| 0.5 * (x - y) / 1e-6)
        .collect();
    dbg!(&diff); */
    /*     println!("M2");
    (&m2_logging.lock().await)
        .chunks()
        .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
        .enumerate()
        .for_each(|(i, x)| println!("{:2}{:+6.3?}", i, x)); */
    /*     println!("AGWS GS");
    (&agws_logging.lock().await)
        .chunks()
        .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
        .enumerate()
        .for_each(|(i, x)| println!("{:2}{:+6.3?}", i, x)); */

    let mut sa: SlopesArray = (&mut *matrix.lock().await).into();
    sa.trim(vec![38, 41]);
    let mat = sa.interaction_matrix();
    println!("{:.3}", mat);
    let matfile = MatFile::save(data_repo.join("closed-loop_DFS.mat"))?;
    matfile.var("dfs_m1_rbm", mat)?;

    Ok(())
}
