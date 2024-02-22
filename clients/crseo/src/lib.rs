/*!
#  CEO Optical Model

This module is a high-level interface to [crseo] and [crseo] is a Rust wrapper around CEO.
CEO is a CUDA-based optical propagation model for the GMT.

Follow the instructions [here](https://github.com/rconan/crseo) to install and to setup CEO.

A default optical model consists in the GMT and an on-axis source
```
use dos_actors::prelude::*;
use gmt_dos_clients_crseo::OpticalModel;
let optical_model = OpticalModel::builder().build().expect("Failed to build CEO optical model");
```
 */

/* pub(crate) mod optical_model;
pub use optical_model::{
    OpticalModel, OpticalModelBuilder, OpticalModelOptions, PSSnOptions, ShackHartmannOptions,
};
pub(crate) mod shackhartmann;

mod sensor;
pub use sensor::SensorBuilder;
*/

use std::{
    ops::{Deref, DerefMut, Mul},
    sync::Arc,
};

pub use crseo::{self, CrseoError};
use interface::{Data, Read, UniqueIdentifier, Update, Write};

mod error;
pub use error::{CeoError, Result};

mod ngao;
pub use ngao::{
    DetectorFrame, GuideStar, OpticalModel, OpticalModelBuilder, ResidualM2modes,
    ResidualPistonMode, WavefrontSensor,
};

mod wavefront_stats;
pub use wavefront_stats::WavefrontStats;

mod pyramid;
pub use pyramid::{PyramidCalibrator, PyramidCommand, PyramidMeasurements, PyramidProcessor};

pub trait Processing {
    type ProcessorData;
    fn processing(&self) -> Self::ProcessorData;
}

/// Sensor data processor
#[derive(Default, Debug)]
pub struct Processor<P: Processing>(P);

impl<P: Processing> From<P> for Processor<P> {
    fn from(value: P) -> Self {
        Processor(value)
    }
}

impl<P: Processing> Deref for Processor<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P: Processing> DerefMut for Processor<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P: Processing + Send + Sync> Update for Processor<P> {
    // fn update(&mut self) {
    //     self.processing();
    // }
}

impl Read<DetectorFrame<f32>> for Processor<PyramidProcessor<f32>> {
    fn read(&mut self, data: Data<DetectorFrame<f32>>) {
        self.frame = data.as_arc();
    }
}

impl<P, T> Write<T> for Processor<P>
where
    P: Processing + Send + Sync,
    T: UniqueIdentifier<DataType = P::ProcessorData>,
{
    fn write(&mut self) -> Option<Data<T>> {
        let data: <P as Processing>::ProcessorData = self.processing();
        Some(Data::new(data))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CalibratingError {
    #[error("crseo error")]
    Crseo(#[from] CrseoError),
}

/// Sensor calibration interface
pub trait Calibrating {
    type ProcessorData: Default;
    type Output;
    // type Calibrator;
    // fn calibrating(&self) -> Result<Self::Calibrator, CalibratingError>;
}

/// Sensor calibration
pub struct Calibration<C: Calibrating> {
    calibrator: C,
    output: Arc<C::Output>,
}

impl<C: Calibrating> Deref for Calibration<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.calibrator
    }
}

impl<C: Calibrating> DerefMut for Calibration<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.calibrator
    }
}

impl<C: Calibrating + Send + Sync> Update for Calibration<C>
where
    <C as Calibrating>::ProcessorData: Sync + Send,
    <C as Calibrating>::Output: Send + Sync,
    // for<'a> &'a C: Mul<&'a C::ProcessorData, Output = ()>,
{
    // fn update(&mut self) {
    //     &self.calibrator * &self.data
    // }
}

impl<C: Calibrating + Send + Sync, T: UniqueIdentifier<DataType = C::ProcessorData>> Read<T>
    for Calibration<C>
where
    <C as Calibrating>::ProcessorData: Send + Sync,
    <C as Calibrating>::Output: Send + Sync,
    for<'a> &'a C: Mul<&'a C::ProcessorData, Output = <C as Calibrating>::Output>,
{
    fn read(&mut self, data: Data<T>) {
        let value = data.as_arc();
        self.output = Arc::new(&self.calibrator * &value);
    }
}

impl<C: Calibrating + Send + Sync, T: UniqueIdentifier<DataType = C::Output>> Write<T>
    for Calibration<C>
where
    <C as Calibrating>::ProcessorData: Send + Sync,
    <C as Calibrating>::Output: Send + Sync,
{
    fn write(&mut self) -> Option<Data<T>> {
        Some(Data::from(&self.output))
    }
}

/*

#[cfg(feature = "crseo")]
/// GMT M1 & M2 state
#[derive(UID)]
#[uid(data = "crseo::gmt::SegmentsDof")]
pub enum GmtState {}
pub enum PointingError {}
impl UniqueIdentifier for PointingError {
    type DataType = (f64, f64);
}

#[cfg(feature = "fem")]
impl<S> dos_actors::io::Write<M1modes> for fem::dos::DiscreteModalSolver<S>
where
    S: fem::dos::Solver + Default,
{
    fn write(&mut self) -> Option<std::sync::Arc<dos_actors::io::Data<M1modes>>> {
        let mut data: std::sync::Arc<dos_actors::io::Data<fem::dos::M1SegmentsAxialD>> =
            self.write()?;
        let inner = std::sync::Arc::get_mut(&mut data)?;
        Some(std::sync::Arc::new(inner.into()))
    }
}
 */
