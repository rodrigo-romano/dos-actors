use std::{sync::Arc, time::Instant};

use crseo::{
    wavefrontsensor::{PhaseSensor, PistonSensor, SegmentCalibration},
    AtmosphereBuilder, Builder, FromBuilder, Gmt, SegmentWiseSensorBuilder, WavefrontSensorBuilder,
};
use gmt_dos_actors::{model::Unknown, prelude::*};
use gmt_dos_clients::{
    interface::{Data, Read, Update, Write},
    Logging, Pulse, Sampler, Tick, Timer,
};
use gmt_dos_clients_arrow::Arrow;
use gmt_dos_clients_crseo::{M2modes, SegmentPiston, SegmentWfeRms, WfeRms};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_m2::asm::segment::{FaceSheetFigure, ModalCommand};
use ngao::{
    GuideStar, LittleOpticalModel, PwfsIntegrator, ResidualM2modes, ResidualPistonMode, SensorData,
    WavefrontSensor,
};
use tokio::sync::Mutex;

/// ASM segment modes dispatcher
pub struct AsmsDispatch {
    n_mode: usize,
    m2_modes: Arc<Data<M2modes>>,
}

impl AsmsDispatch {
    pub fn new(n_mode: usize) -> Self {
        Self {
            n_mode,
            m2_modes: Arc::new(Data::new(vec![])),
        }
    }
}

impl Update for AsmsDispatch {}

impl Read<M2modes> for AsmsDispatch {
    fn read(&mut self, data: Arc<Data<M2modes>>) {
        self.m2_modes = Arc::clone(&data);
    }
}

impl<const ID: u8> Write<ModalCommand<ID>> for AsmsDispatch {
    fn write(&mut self) -> Option<Arc<Data<ModalCommand<ID>>>> {
        let data = self
            .m2_modes
            .chunks(self.n_mode)
            .nth(ID as usize - 1)
            .expect(&format!("failed to retrieve ASM #{ID} modes"));
        Some(Arc::new(Data::new(data.to_vec())))
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
    /// Build a new NGAO control system
    pub async fn build(
        self,
        n_sample: usize,
        sampling_frequency: f64,
        asm_dispatch: &mut Actor<AsmsDispatch>,
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
        slopes_mat.pseudo_inverse().unwrap();

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
        piston_mat.pseudo_inverse().unwrap();
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
        let mut timer: Initiator<_> = Timer::new(n_sample).into();

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
        let mut sampler_pwfs_to_plant: Actor<_, PYWFS, 1> = (
            Sampler::default(),
            "ZOH transition:
    PWFS -> ASMS",
        )
            .into();

        let mut pwfs_integrator: Actor<_, PYWFS, PYWFS> = (
            PwfsIntegrator::single_single(self.n_mode, 0.5f64),
            "PWFS
    Integrator",
        )
            .into();
        /*
        let mut debug: Terminator<_, PYWFS> = (
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
            .into_input(&mut sampler_pwfs_to_plant)?;
        sampler_pwfs_to_plant
            .add_output()
            .build::<M2modes>()
            .into_input(asm_dispatch)?;

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
                sampler_hdfs_to_pwfs,
                sampler_pwfs_to_plant
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
