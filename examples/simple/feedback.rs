use std::sync::Arc;

use crate::FilterToCompensator;
use dos_actors::{
    io::{Data, Read, Write},
    Update,
};

#[derive(Default)]
pub struct Compensator(f64, f64);
impl Update for Compensator {}
impl Read<f64, FilterToCompensator> for Compensator {
    fn read(&mut self, data: Arc<Data<f64, FilterToCompensator>>) {
        self.0 = **data;
    }
}
impl Read<f64, IntegratorToCompensator> for Compensator {
    fn read(&mut self, data: Arc<Data<f64, IntegratorToCompensator>>) {
        self.1 = **data;
    }
}
pub enum CompensatorToIntegrator {}
impl Write<f64, CompensatorToIntegrator> for Compensator {
    fn write(&self) -> Option<Arc<Data<f64, CompensatorToIntegrator>>> {
        Some(Arc::new(Data::new(self.0 - self.1)))
    }
}

#[derive(Default)]
pub struct Integrator {
    gain: f64,
    mem: Vec<f64>,
}
impl Integrator {
    pub fn new(gain: f64, n_data: usize) -> Self {
        Self {
            gain,
            mem: vec![0f64; n_data],
        }
    }
    pub fn last(&self) -> Option<Vec<f64>> {
        Some(self.mem.clone())
    }
}
impl Update for Integrator {}
impl Read<f64, CompensatorToIntegrator> for Integrator {
    fn read(&mut self, data: Arc<Data<f64, CompensatorToIntegrator>>) {
        self.mem[0] += **data * self.gain;
    }
}
pub enum IntegratorToCompensator {}
impl Write<f64, IntegratorToCompensator> for Integrator {
    fn write(&self) -> Option<Arc<Data<f64, IntegratorToCompensator>>> {
        self.last().map(|x| Arc::new(Data::new(x[0])))
    }
}
