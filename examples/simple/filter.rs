use crate::SignalToFilter;
use dos_actors::{
    io::{Consuming, Data, Producing},
    Updating,
};
use rand_distr::{Distribution, Normal};
use std::sync::Arc;

pub struct Filter {
    data: f64,
    noise: Normal<f64>,
    step: usize,
}
impl Default for Filter {
    fn default() -> Self {
        Self {
            data: 0f64,
            noise: Normal::new(0.3, 0.05).unwrap(),
            step: 0,
        }
    }
}
impl Updating for Filter {
    fn update(&mut self) {
        self.data += 0.05
            * (2. * std::f64::consts::PI * self.step as f64 * (1e3f64 * 2e-2).recip()).sin()
            + self.noise.sample(&mut rand::thread_rng());
        self.step += 1;
    }
}
impl Consuming<f64, SignalToFilter> for Filter {
    fn consume(&mut self, data: Arc<Data<f64, SignalToFilter>>) {
        self.data = **data;
    }
}
#[derive(Debug)]
pub enum FilterToSink {}
impl Producing<f64, FilterToSink> for Filter {
    fn produce(&self) -> Option<Arc<Data<f64, FilterToSink>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}
#[derive(Debug)]
pub enum FilterToSampler {}
impl Producing<f64, FilterToSampler> for Filter {
    fn produce(&self) -> Option<Arc<Data<f64, FilterToSampler>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}
#[derive(Debug)]
pub enum FilterToCompensator {}
impl Producing<f64, FilterToCompensator> for Filter {
    fn produce(&self) -> Option<Arc<Data<f64, FilterToCompensator>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}
