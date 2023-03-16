use std::sync::{Arc, Mutex};

use crseo::{
    wavefrontsensor::{Calibration, DataRef, Slopes},
    SegmentWiseSensor, Source,
};
use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update, Write};
use gmt_dos_clients_crseo::M2modes;

mod optical_model;
pub use optical_model::LittleOpticalModel;
pub enum GuideStar {}
impl UniqueIdentifier for GuideStar {
    type DataType = Arc<Mutex<Source>>;
}

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
    fn read(&mut self, data: Arc<Data<GuideStar>>) {
        let src = &mut (*data.lock().unwrap());
        self.sensor.propagate(src);
    }
}

impl<T: SegmentWiseSensor> Write<M2modes> for WavefrontSensor<T> {
    fn write(&mut self) -> Option<Arc<Data<M2modes>>> {
        self.sensor.transform(&self.calib).map(|x| {
            self.sensor.reset();
            Arc::new(Data::new(x.into_iter().map(|x| x as f64).collect()))
        })
    }
}

pub enum Piston {}
impl UniqueIdentifier for Piston {
    type DataType = Vec<f64>;
}
impl<T: SegmentWiseSensor> Write<Piston> for WavefrontSensor<T>
where
    Slopes: for<'a> From<(&'a DataRef, &'a T)>,
{
    fn write(&mut self) -> Option<Arc<Data<Piston>>> {
        let s: Vec<_> = self
            .calib
            .iter()
            .map(|s| Slopes::from((&s.data_ref, &self.sensor)))
            .flat_map(|s| Vec::<f32>::from(s))
            .map(|x| x as f64)
            .collect();
        Some(Arc::new(Data::new(s)))
    }
}
