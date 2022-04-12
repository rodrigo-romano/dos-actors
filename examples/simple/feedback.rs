use std::sync::Arc;

use crate::FilterToDifferentiator;
use dos_actors::{
    io::{Data, Read, Write},
    Update,
};

#[derive(Default)]
pub struct Differentiator(f64, f64);
impl Update for Differentiator {}
impl Read<f64, FilterToDifferentiator> for Differentiator {
    fn read(&mut self, data: Arc<Data<f64, FilterToDifferentiator>>) {
        self.0 = **data;
    }
}
impl Read<f64, IntegratorToDifferentiator> for Differentiator {
    fn read(&mut self, data: Arc<Data<f64, IntegratorToDifferentiator>>) {
        self.1 = **data;
    }
}
pub enum DifferentiatorToIntegrator {}
impl Write<f64, DifferentiatorToIntegrator> for Differentiator {
    fn write(&mut self) -> Option<Arc<Data<f64, DifferentiatorToIntegrator>>> {
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
impl Read<f64, DifferentiatorToIntegrator> for Integrator {
    fn read(&mut self, data: Arc<Data<f64, DifferentiatorToIntegrator>>) {
        self.mem[0] += **data * self.gain;
    }
}
pub enum IntegratorToDifferentiator {}
impl Write<f64, IntegratorToDifferentiator> for Integrator {
    fn write(&mut self) -> Option<Arc<Data<f64, IntegratorToDifferentiator>>> {
        self.last().map(|x| Arc::new(Data::new(x[0])))
    }
}
