use std::{
    ops::Mul,
    sync::{Arc, Mutex},
};

use crseo::{
    wavefrontsensor::{Calibration, DataRef, Slopes},
    Propagation, SegmentWiseSensor, Source,
};
use interface::{Data, Read, UniqueIdentifier, Update, Write, UID};

pub enum GuideStar {}
impl UniqueIdentifier for GuideStar {
    type DataType = Arc<Mutex<Source>>;
}

#[derive(UID)]
pub enum PistonMode {}

#[derive(UID)]
#[uid(data = Vec<f32>)]
pub enum SensorData {}

pub struct WavefrontSensor<T> {
    sensor: T,
    calib: Calibration,
}
impl<T: SegmentWiseSensor> WavefrontSensor<T> {
    pub fn new(sensor: T, calib: Calibration) -> Self {
        Self { sensor, calib }
    }
}

impl<T> Update for WavefrontSensor<T> {}

impl<T: SegmentWiseSensor> Read<GuideStar> for WavefrontSensor<T> {
    fn read(&mut self, data: Data<GuideStar>) {
        let src = &mut (*data.lock().unwrap());
        self.sensor.propagate(src);
    }
}

impl<T, U> Write<U> for WavefrontSensor<T>
where
    for<'a> &'a Calibration: Mul<&'a T, Output = Option<Vec<f32>>>,
    T: SegmentWiseSensor,
    U: UniqueIdentifier<DataType = Vec<f64>>,
{
    fn write(&mut self) -> Option<Data<U>> {
        (&self.calib * &self.sensor).map(|x| {
            self.sensor.reset();
            Data::new(x.into_iter().map(|x| x as f64).collect())
        })
    }
}

impl<T> Write<SensorData> for WavefrontSensor<T>
where
    for<'a> Slopes: From<(&'a DataRef, &'a T)>,
    T: SegmentWiseSensor,
{
    fn write(&mut self) -> Option<Data<SensorData>> {
        let data: Vec<f32> = self
            .calib
            .iter()
            .map(|s| Slopes::from((&s.data_ref, &self.sensor)))
            .flat_map(|s| Vec::<f32>::from(s))
            .collect();
        // self.sensor.reset();
        Some(Data::new(data))
    }
}

pub struct ShackHartmann(pub crseo::ShackHartmann<crseo::Diffractive>);

impl Update for ShackHartmann {}

impl Read<GuideStar> for ShackHartmann {
    fn read(&mut self, data: Data<GuideStar>) {
        let src = &mut (*data.lock().unwrap());
        self.0.propagate(src);
    }
}

#[derive(UID)]
#[uid(data = Vec<f32>)]
pub enum Frame {}

impl Write<Frame> for ShackHartmann {
    fn write(&mut self) -> Option<Data<Frame>> {
        <crseo::ShackHartmann<crseo::Diffractive> as crseo::WavefrontSensor>::frame(&self.0).map(
            |frame| {
                <crseo::ShackHartmann<crseo::Diffractive> as crseo::WavefrontSensor>::reset(
                    &mut self.0,
                );
                frame.into()
            },
        )
    }
}
