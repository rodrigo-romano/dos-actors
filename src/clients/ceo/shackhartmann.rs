use super::optical_model::{Result, SensorBuilder, SensorFn};
use super::OpticalModel;
use crate::io::{Data, Write};
use crseo::{
    shackhartmann::Model, Builder, ShackHartmannBuilder, WavefrontSensor, WavefrontSensorBuilder,
    GMT, SOURCE,
};
use nalgebra as na;
use std::sync::Arc;

impl<M> SensorBuilder for ShackHartmannBuilder<M>
where
    M: 'static + Model,
{
    fn build(
        self,
        gmt_builder: GMT,
        src_builder: SOURCE,
        threshold: f64,
    ) -> Result<Box<dyn WavefrontSensor>> {
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
        let mut sensor = Builder::build(self).unwrap();
        let mut gmt = gmt_builder.build()?;
        src.reset();
        src.through(&mut gmt);
        sensor.set_valid_lenslet(&valid_lenslets);
        sensor.set_reference_slopes(&mut src);
        Ok(Box::new(sensor))
    }
}

impl Write<Vec<f64>, super::SensorData> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::SensorData>>> {
        if let Some(sensor) = &mut self.sensor {
            (*sensor).readout();
            self.frame = (*sensor).frame();
            (*sensor).process();
            let data: Vec<f64> = (*sensor).data();
            (*sensor).reset();
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
impl Write<Vec<f64>, crate::clients::fsm::TTFB> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<crate::clients::fsm::TTFB>>> {
        if let Some(sensor) = &mut self.sensor {
            (*sensor).readout();
            self.frame = (*sensor).frame();
            (*sensor).process();
            let data: Vec<f64> = (*sensor).data();
            (*sensor).reset();
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
impl Write<Vec<f32>, super::DetectorFrame> for OpticalModel {
    fn write(&mut self) -> Option<Arc<Data<super::DetectorFrame>>> {
        if let Some(sensor) = &mut self.sensor {
            if self.frame.is_none() {
                self.frame = sensor.frame();
            }
            if let Some(frame) = &self.frame {
                Some(Arc::new(Data::new(frame.to_vec())))
            } else {
                None
            }
        } else {
            None
        }
    }
}
