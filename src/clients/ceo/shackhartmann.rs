use super::optical_model::{Result, SensorBuilder, SensorFn};
use super::OpticalModel;
use crate::io::{Data, Write};
use crseo::{
    shackhartmann::Model, Builder, Diffractive, Geometric, ShackHartmann, ShackHartmannBuilder,
    WavefrontSensor, WavefrontSensorBuilder, GMT, SH24, SH48, SOURCE,
};
use nalgebra as na;
use std::sync::Arc;

impl<M: Model> SensorBuilder for ShackHartmannBuilder<M> {
    type Sensor = ShackHartmann<M>;
    fn build(self, gmt_builder: GMT, src_builder: SOURCE, threshold: f64) -> Result<Self::Sensor> {
        let mut src = self.guide_stars(Some(src_builder)).build()?;
        let n_side_lenslet = self.lenslet_array.0;
        let n = n_side_lenslet.pow(2) * self.n_sensor;
        let mut valid_lenslets: Vec<i32> = (1..=7).fold(vec![0i32; n as usize], |mut a, sid| {
            let mut gmt = gmt_builder.clone().build().unwrap();
            src.reset();
            src.through(gmt.keep(&mut [sid])).xpupil();
            let mut sensor = Builder::build(self.clone()).unwrap();
            sensor.calibrate(&mut src, threshold);
            let valid_lenslets: Vec<f32> = sensor.lenslet_mask().into();
            /*valid_lenslets.chunks(48).for_each(|row| {
                row.iter().for_each(|val| print!("{val:.2},"));
                println!("");
            });
            println!("");*/
            a.iter_mut()
                .zip(&valid_lenslets)
                .filter(|(_, v)| **v > 0.)
                .for_each(|(a, _)| {
                    *a += 1;
                });
            a
        });
        /*
        valid_lenslets.chunks(48).for_each(|row| {
            row.iter().for_each(|val| print!("{val}"));
            println!("");
        });*/
        valid_lenslets
            .iter_mut()
            .filter(|v| **v > 1)
            .for_each(|v| *v = 0);
        //dbg!(valid_lenslets.iter().cloned().sum::<i32>());
        let mut sensor = Builder::build(self.clone()).unwrap();
        let mut gmt = gmt_builder.clone().build()?;
        src.reset();
        src.through(&mut gmt);
        sensor.set_valid_lenslet(&valid_lenslets);
        sensor.set_reference_slopes(&mut src);
        Ok(sensor)
    }
}
impl<M: Model> SensorBuilder for SH24<M> {
    type Sensor = ShackHartmann<M>;
    fn build(self, gmt_builder: GMT, src_builder: SOURCE, threshold: f64) -> Result<Self::Sensor> {
        let mut src = self.guide_stars(Some(src_builder)).build()?;
        let n_side_lenslet = self.lenslet_array.0;
        let n = n_side_lenslet.pow(2) * self.n_sensor;
        let mut valid_lenslets: Vec<i32> = (1..=7).fold(vec![0i32; n as usize], |mut a, sid| {
            let mut gmt = gmt_builder.clone().build().unwrap();
            src.reset();
            src.through(gmt.keep(&mut [sid])).xpupil();
            let mut sensor = Builder::build(self.clone()).unwrap();
            sensor.calibrate(&mut src, threshold);
            let valid_lenslets: Vec<f32> = sensor.lenslet_mask().into();
            /*valid_lenslets.chunks(48).for_each(|row| {
                row.iter().for_each(|val| print!("{val:.2},"));
                println!("");
            });
            println!("");*/
            a.iter_mut()
                .zip(&valid_lenslets)
                .filter(|(_, v)| **v > 0.)
                .for_each(|(a, _)| {
                    *a += 1;
                });
            a
        });
        /*
        valid_lenslets.chunks(48).for_each(|row| {
            row.iter().for_each(|val| print!("{val}"));
            println!("");
        });*/
        valid_lenslets
            .iter_mut()
            .filter(|v| **v > 1)
            .for_each(|v| *v = 0);
        let mut sensor = Builder::build(self.clone()).unwrap();
        let mut gmt = gmt_builder.clone().build()?;
        src.reset();
        src.through(&mut gmt);
        sensor.set_valid_lenslet(&valid_lenslets);
        sensor.set_reference_slopes(&mut src);
        Ok(sensor)
    }
}
impl<M: Model> SensorBuilder for SH48<M> {
    type Sensor = ShackHartmann<M>;
    fn build(self, gmt_builder: GMT, src_builder: SOURCE, threshold: f64) -> Result<Self::Sensor> {
        let mut src = self.guide_stars(Some(src_builder)).build()?;
        let n_side_lenslet = self.lenslet_array.0;
        let n = n_side_lenslet.pow(2) * self.n_sensor;
        let mut valid_lenslets: Vec<i32> = (1..=7).fold(vec![0i32; n as usize], |mut a, sid| {
            let mut gmt = gmt_builder.clone().build().unwrap();
            src.reset();
            src.through(gmt.keep(&mut [sid])).xpupil();
            let mut sensor = Builder::build(self.clone()).unwrap();
            sensor.calibrate(&mut src, threshold);
            let valid_lenslets: Vec<f32> = sensor.lenslet_mask().into();
            /*valid_lenslets.chunks(48).for_each(|row| {
                row.iter().for_each(|val| print!("{val:.2},"));
                println!("");
            });
            println!("");*/
            a.iter_mut()
                .zip(&valid_lenslets)
                .filter(|(_, v)| **v > 0.)
                .for_each(|(a, _)| {
                    *a += 1;
                });
            a
        });
        /*
        valid_lenslets.chunks(48).for_each(|row| {
            row.iter().for_each(|val| print!("{val}"));
            println!("");
        });*/
        valid_lenslets
            .iter_mut()
            .filter(|v| **v > 1)
            .for_each(|v| *v = 0);
        let mut sensor = Builder::build(self.clone()).unwrap();
        let mut gmt = gmt_builder.clone().build()?;
        src.reset();
        src.through(&mut gmt);
        sensor.set_valid_lenslet(&valid_lenslets);
        sensor.set_reference_slopes(&mut src);
        Ok(sensor)
    }
}

impl Write<Vec<f64>, super::SensorData>
    for OpticalModel<ShackHartmann<Geometric>, ShackHartmannBuilder<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.readout().process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(
                    data.into_iter().map(|x| x as f64).collect(),
                ))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data
                    .into_iter()
                    .map(|x| x as f64)
                    .collect())))),
                SensorFn::Matrix(mat) => {
                    let u: Vec<_> = data.into_iter().map(|x| x as f64).collect();
                    let v = na::DVector::from_vec(u);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
impl Write<Vec<f64>, super::SensorData>
    for OpticalModel<ShackHartmann<Diffractive>, ShackHartmannBuilder<Diffractive>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            self.frame = Some(sensor.readout().frame());
            let data: Vec<f64> = sensor.process().data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(
                    data.into_iter().map(|x| x as f64).collect(),
                ))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data
                    .into_iter()
                    .map(|x| x as f64)
                    .collect())))),
                SensorFn::Matrix(mat) => {
                    let u: Vec<_> = data.into_iter().map(|x| x as f64).collect();
                    let v = na::DVector::from_vec(u);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
impl Write<Vec<f32>, super::DetectorFrame>
    for OpticalModel<ShackHartmann<Diffractive>, ShackHartmannBuilder<Diffractive>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f32>, super::DetectorFrame>>> {
        if let Some(sensor) = &mut self.sensor {
            let frame = self.frame.get_or_insert(sensor.frame());
            Some(Arc::new(Data::new(frame.to_vec())))
        } else {
            None
        }
    }
}
impl Write<Vec<f32>, super::DetectorFrame>
    for OpticalModel<ShackHartmann<Diffractive>, SH24<Diffractive>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f32>, super::DetectorFrame>>> {
        if let Some(sensor) = &mut self.sensor {
            let frame = self.frame.get_or_insert(sensor.frame());
            Some(Arc::new(Data::new(frame.to_vec())))
        } else {
            None
        }
    }
}
impl Write<Vec<f32>, super::DetectorFrame>
    for OpticalModel<ShackHartmann<Diffractive>, SH48<Diffractive>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f32>, super::DetectorFrame>>> {
        if let Some(sensor) = &mut self.sensor {
            let frame = self.frame.get_or_insert(sensor.frame());
            Some(Arc::new(Data::new(frame.to_vec())))
        } else {
            None
        }
    }
}
impl Write<Vec<f64>, super::SensorData>
    for OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
#[cfg(feature = "fsm")]
impl Write<Vec<f64>, crate::clients::fsm::TTFB>
    for OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
#[cfg(feature = "fsm")]
impl Write<Vec<f64>, crate::clients::fsm::TTFB>
    for OpticalModel<ShackHartmann<Diffractive>, SH24<Diffractive>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.readout().process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
impl Write<Vec<f64>, super::SensorData>
    for OpticalModel<ShackHartmann<Geometric>, SH48<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
impl Write<Vec<f64>, super::SensorData>
    for OpticalModel<ShackHartmann<Diffractive>, SH48<Diffractive>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            self.frame = Some(sensor.readout().frame());
            let data: Vec<f64> = sensor.process().data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
#[cfg(feature = "fsm")]
impl Write<Vec<f64>, crate::clients::fsm::TTFB>
    for OpticalModel<ShackHartmann<Geometric>, SH48<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
#[cfg(features = "fsm")]
impl Write<Vec<f64>, crate::clients::fsm::TTFB>
    for OpticalModel<ShackHartmann<Geometric>, SH24<Geometric>>
{
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            sensor.process();
            let data: Vec<f64> = sensor.data().into();
            sensor.reset();
            match &self.sensor_fn {
                SensorFn::None => Some(Arc::new(Data::new(data))),
                SensorFn::Fn(f) => Some(Arc::new(Data::new(f(data)))),
                SensorFn::Matrix(mat) => {
                    let v = na::DVector::from_vec(data);
                    let y = mat * v;
                    Some(Arc::new(Data::new(y.as_slice().to_vec())))
                }
            }
        } else {
            None
        }
    }
}
