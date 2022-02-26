use std::sync::Arc;

use crate::FilterToCompensator;
use dos_actors::{
    io::{Consuming, Data, Producing},
    Updating,
};

#[derive(Default)]
pub struct Compensator(f64, f64);
impl Updating for Compensator {}
impl Consuming<f64, FilterToCompensator> for Compensator {
    fn consume(&mut self, data: Arc<Data<f64, FilterToCompensator>>) {
        self.0 = **data;
    }
}
impl Consuming<f64, IntegratorToCompensator> for Compensator {
    fn consume(&mut self, data: Arc<Data<f64, IntegratorToCompensator>>) {
        self.1 = **data;
    }
}
pub enum CompensatorToIntegrator {}
impl Producing<f64, CompensatorToIntegrator> for Compensator {
    fn produce(&self) -> Option<Arc<Data<f64, CompensatorToIntegrator>>> {
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
impl Updating for Integrator {}
impl Consuming<f64, CompensatorToIntegrator> for Integrator {
    fn consume(&mut self, data: Arc<Data<f64, CompensatorToIntegrator>>) {
        self.mem[0] += **data * self.gain;
    }
}
pub enum IntegratorToCompensator {}
impl Producing<f64, IntegratorToCompensator> for Integrator {
    fn produce(&self) -> Option<Arc<Data<f64, IntegratorToCompensator>>> {
        self.last().map(|x| Arc::new(Data::new(x[0])))
    }
}
