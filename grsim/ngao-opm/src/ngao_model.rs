use std::{sync::Arc, time::Instant};

use crseo::{
    wavefrontsensor::{PhaseSensor, PistonSensor, SegmentCalibration},
    AtmosphereBuilder, Builder, FromBuilder, Gmt, SegmentWiseSensorBuilder, WavefrontSensorBuilder,
};
use gmt_dos_actors::{model::Unknown, prelude::*};
use gmt_dos_clients::{
    interface::{Data, Read, Update, Write},
    Logging, Pulse, Tick, Timer,
};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_crseo::{M2modes, SegmentPiston, SegmentWfeRms, WfeRms};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{AsmCommand, FaceSheetFigure};
use gmt_dos_clients_m2_ctrl::Preprocessor;
use nalgebra::{DMatrix, DVector};
use ngao::{
    GuideStar, LittleOpticalModel, PwfsIntegrator, ResidualM2modes, ResidualPistonMode, SensorData,
    WavefrontSensor,
};
use tokio::sync::Mutex;

/// ASM segment modes dispatcher
pub struct AsmsDispatch {
    n_mode: usize,
    m2_modes: Data<M2modes>,
    modes2positions: Option<Vec<DMatrix<f64>>>,
    m2_modes_offset: Vec<Option<Arc<Vec<f64>>>>,
    prep: Option<Preprocessor>,
    data: Option<Vec<Arc<Vec<f64>>>>,
}

impl AsmsDispatch {
    pub fn new(
        n_mode: usize,
        modes2positions: Option<Vec<DMatrix<f64>>>,
        prep: Option<Preprocessor>,
    ) -> Self {
        Self {
            n_mode,
            m2_modes: Vec::new().into(),
            modes2positions,
            m2_modes_offset: vec![None; 7],
            prep,
            data: None,
        }
    }
}

impl Update for AsmsDispatch {
    fn update(&mut self) {
        for (i, segment_modes) in self.m2_modes.chunks(self.n_mode).enumerate() {
            let mut data = segment_modes.to_vec();
            if let Some(offset) = &self.m2_modes_offset[i] {
                data.iter_mut()
                    .zip(offset.iter())
                    .for_each(|(d, &o)| *d += o);
            }
            if let Some(modes2positions) = &self.modes2positions {
                let m = &modes2positions[i];
                let c = m * DVector::<f64>::from_column_slice(&data);
                data = c.as_slice().to_vec();
                if i == 6 {
                    self.prep.as_ref().map(|prep| prep.apply(&mut data));
                }
            }
            let cmd = self.data.get_or_insert(vec![]);
            if let Some(segment_cmd) = cmd.get_mut(i) {
                *segment_cmd = Arc::new(data);
            } else {
                cmd.push(Arc::new(data))
            }
        }
    }
}

impl Read<M2modes> for AsmsDispatch {
    fn read(&mut self, data: Data<M2modes>) {
        self.m2_modes = data;
    }
}

impl<const ID: u8> Read<AsmCommand<ID>> for AsmsDispatch {
    fn read(&mut self, data: Data<AsmCommand<ID>>) {
        self.m2_modes_offset[ID as usize - 1] = Some(data.as_arc());
    }
}

impl<const ID: u8> Write<AsmCommand<ID>> for AsmsDispatch {
    fn write(&mut self) -> Option<Data<AsmCommand<ID>>> {
        self.data
            .as_ref()
            .and_then(|data| data.get(ID as usize - 1))
            .map(|x| x.clone().into())
    }
}

pub enum PistonCapture {
    HDFS,
    PWFS,
    Bound(f64),
}
impl PistonCapture {
    pub fn bound(&self) -> f64 {
        match self {
            PistonCapture::HDFS => f64::NEG_INFINITY,
            PistonCapture::PWFS => f64::INFINITY,
            PistonCapture::Bound(value) => *value,
        }
    }
}

/// Buidler for NGAO control system
pub struct NgaoBuilder<const PYWFS: usize, const HDFS: usize> {
    n_mode: usize,
    modes: String,
    n_lenslet: usize,
    n_px_lenslet: usize,
    wrapping: Option<f64>,
    atm_builder: Option<AtmosphereBuilder>,
    piston_capture: PistonCapture,
    integrator_gain: f64,
}

impl<const PYWFS: usize, const HDFS: usize> Default for NgaoBuilder<PYWFS, HDFS> {
    fn default() -> Self {
        Self {
            n_mode: 66,
            modes: String::from("M2_OrthoNorm_KarhunenLoeveModes"),
            n_lenslet: 92,
            n_px_lenslet: 4,
            wrapping: None,
            atm_builder: None,
            piston_capture: PistonCapture::PWFS,
            integrator_gain: 0.5,
        }
    }
}

impl<const PYWFS: usize, const HDFS: usize> NgaoBuilder<PYWFS, HDFS> {
    /// Sets the filename of the .ceo file with the M2 modes
    pub fn modes_src_file<S: Into<String>>(mut self, modes: S) -> Self {
        self.modes = modes.into();
        self
    }
    /// Sets the number of modes
    pub fn n_mode(mut self, n_mode: usize) -> Self {
        self.n_mode = n_mode;
        self
    }
    /// Sets the number of lenslet
    pub fn n_lenslet(mut self, n_lenslet: usize) -> Self {
        self.n_lenslet = n_lenslet;
        self
    }
    /// Sets the number of pixel per lenslet
    pub fn n_px_lenslet(mut self, n_px_lenslet: usize) -> Self {
        self.n_px_lenslet = n_px_lenslet;
        self
    }
    /// Sets the piston wrapping value
    pub fn wrapping(mut self, wrapping: f64) -> Self {
        self.wrapping = Some(wrapping);
        self
    }
    /// Sets the model of the atmospheric turbulence
    pub fn atmosphere(mut self, atm_builder: AtmosphereBuilder) -> Self {
        self.atm_builder = Some(atm_builder);
        self
    }
    pub fn piston_capture(mut self, piston_capture: PistonCapture) -> Self {
        self.piston_capture = piston_capture;
        self
    }
    /// Sets the NGAO integral controller gain
    ///
    /// Per default, the gain is set to 0.5
    pub fn gain(mut self, gain: f64) -> Self {
        self.integrator_gain = gain;
        self
    }
    /// Build a new NGAO control system
    pub async fn build(
        self,
        n_sample: usize,
        sampling_frequency: f64,
        asm_dispatch: &mut Actor<AsmsDispatch, PYWFS, 1>,
        plant: &mut Actor<DiscreteModalSolver<ExponentialMatrix>>,
    ) -> anyhow::Result<(Arc<Mutex<LittleOpticalModel>>, Model<Unknown>)> {
        let builder = if let Some(wrapping) = self.wrapping {
            PhaseSensor::builder()
                .lenslet(self.n_lenslet, self.n_px_lenslet)
                .wrapping(wrapping)
        } else {
            PhaseSensor::builder().lenslet(self.n_lenslet, self.n_px_lenslet)
        };
        let src_builder = builder.guide_stars(None);

        let now = Instant::now();
        let mut slopes_mat = builder.clone().calibrate(
            SegmentCalibration::modes(&self.modes, 0..self.n_mode, "M2"),
            src_builder.clone(),
        );
        println!(
            "M2 {}modes/segment calibrated in {}s",
            self.n_mode,
            now.elapsed().as_secs()
        );
        // MATLAB
        let data_repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        let matfile = matio_rs::MatFile::save(
            data_repo.join(format!("asms_{}_calibration.mat", self.n_mode)),
        )?;
        for (i, mat) in slopes_mat.interaction_matrices().iter().enumerate() {
            matfile.var(format!("asms{}", i + 1), mat)?;
        }
        // MATLAB
        slopes_mat.pseudo_inverse(None).unwrap();

        let piston_builder = PistonSensor::builder().pupil_sampling(builder.pupil_sampling());
        let now = Instant::now();
        let mut piston_mat = piston_builder.calibrate(
            SegmentCalibration::modes(&self.modes, 0..1, "M2"),
            src_builder.clone(),
        );
        println!(
            "M2 {}modes/segment calibrated in {}s",
            1,
            now.elapsed().as_secs()
        );
        piston_mat.pseudo_inverse(None).unwrap();
        let p2m = piston_mat.concat_pinv();
        dbg!(&p2m);

        let gom = if let Some(atm_builder) = self.atm_builder {
            LittleOpticalModel::builder().atmosphere(atm_builder)
        } else {
            LittleOpticalModel::builder()
        }
        .gmt(Gmt::builder().m2(&self.modes, self.n_mode))
        .source(src_builder)
        .sampling_frequency(sampling_frequency)
        .build()?
        .into_arcx();

        let mut gom_act: Actor<_> = Actor::new(gom.clone()).name("GS>>(GMT+ATM)");

        let mut sensor: Actor<_, 1, PYWFS> =
            (WavefrontSensor::new(builder.build()?, slopes_mat), "PWFS").into();
        let mut piston_sensor: Actor<_, 1, HDFS> = (
            WavefrontSensor::new(piston_builder.build()?, piston_mat),
            "HDFS",
        )
            .into();
        let mut timer: Initiator<Timer, 1> = Timer::new(n_sample).into();

        let logging = Arrow::builder(n_sample)
            .filename("ngao.parquet")
            .build()
            .into_arcx();
        let mut logger: Terminator<_> = Actor::new(logging.clone());
        let piston_logging = Logging::new(1).into_arcx();
        let mut piston_logger: Terminator<_, HDFS> = Actor::new(piston_logging.clone()).name(
            "HDFS
    Logger",
        );

        let mut sampler_hdfs_to_pwfs: Actor<_, HDFS, PYWFS> = (
            Pulse::new(1),
            "Pulse transition:
    HDFS -> PWFS",
        )
            .into();
        /*         let mut sampler_pwfs_to_plant: Actor<_, PYWFS, 1> = (
                Sampler::default(),
                "ZOH transition:
        PWFS -> ASMS",
            )
                .into(); */

        let mut pwfs_integrator: Actor<_, PYWFS, PYWFS> = (
            PwfsIntegrator::new(self.n_mode, self.integrator_gain),
            "PWFS
    Integrator",
        )
            .into();

        /*         let mut debug: Terminator<_, PYWFS> = (
            Arrow::builder(n_sample).filename("debug.parquet").build(),
            "Debugger",
        )
            .into(); */

        timer
            .add_output()
            .build::<Tick>()
            .into_input(&mut gom_act)?;
        gom_act
            .add_output()
            .multiplex(2)
            .build::<GuideStar>()
            .into_input(&mut sensor)
            .into_input(&mut piston_sensor)?;
        sensor
            .add_output()
            // .multiplex(2)
            .build::<ResidualM2modes>()
            .into_input(&mut pwfs_integrator)?;
        // .logn(&mut debug, self.n_mode * 7)
        // .await?;
        /*     sampler_pwfs_to_pwfs_ctrl
        .add_output()
        .bootstrap()
        .build::<ResidualM2modes>()
        .into_input(&mut pwfs_integrator)?; */
        gom_act
            .add_output()
            .unbounded()
            .build::<WfeRms>()
            .log(&mut logger)
            .await?;
        gom_act
            .add_output()
            .unbounded()
            .build::<SegmentWfeRms>()
            .log(&mut logger)
            .await?;
        gom_act
            .add_output()
            .unbounded()
            .build::<SegmentPiston>()
            .log(&mut logger)
            .await?;
        piston_sensor
            .add_output()
            .bootstrap()
            .unbounded()
            .build::<SensorData>()
            .into_input(&mut piston_logger)?;
        piston_sensor
            .add_output()
            .bootstrap()
            .build::<ResidualPistonMode>()
            .into_input(&mut sampler_hdfs_to_pwfs)?;
        sampler_hdfs_to_pwfs
            .add_output()
            // .bootstrap()
            .build::<ResidualPistonMode>()
            .into_input(&mut pwfs_integrator)?;
        pwfs_integrator
            .add_output()
            .bootstrap()
            .build::<M2modes>()
            .into_input(asm_dispatch)?;
        /*         sampler_pwfs_to_plant
        .add_output()
        .build::<M2modes>()
        .into_input(asm_dispatch)?; */

        plant
            .add_output()
            .bootstrap()
            .build::<FaceSheetFigure<1>>()
            .into_input(&mut gom_act)?;
        plant
            .add_output()
            .bootstrap()
            .build::<FaceSheetFigure<2>>()
            .into_input(&mut gom_act)?;
        plant
            .add_output()
            .bootstrap()
            .build::<FaceSheetFigure<3>>()
            .into_input(&mut gom_act)?;
        plant
            .add_output()
            .bootstrap()
            .build::<FaceSheetFigure<4>>()
            .into_input(&mut gom_act)?;
        plant
            .add_output()
            .bootstrap()
            .build::<FaceSheetFigure<5>>()
            .into_input(&mut gom_act)?;
        plant
            .add_output()
            .bootstrap()
            .build::<FaceSheetFigure<6>>()
            .into_input(&mut gom_act)?;
        plant
            .add_output()
            .bootstrap()
            .build::<FaceSheetFigure<7>>()
            .into_input(&mut gom_act)?;

        Ok((
            gom,
            model!(
                timer,
                gom_act,
                sensor,
                piston_sensor,
                logger,
                piston_logger,
                pwfs_integrator,
                sampler_hdfs_to_pwfs // sampler_pwfs_to_plant,
                                     // debug
            )
            .name("NGAO")
            .flowchart(),
        ))
    }
}

/// NGAO control system
pub struct Ngao<const PYWFS: usize, const HDFS: usize> {}

impl<const PYWFS: usize, const HDFS: usize> Ngao<PYWFS, HDFS> {
    /// Creates a default builder for NGAO control systems
    pub fn builder() -> NgaoBuilder<PYWFS, HDFS> {
        Default::default()
    }
}
