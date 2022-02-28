use crate::SignalToFilter;
use dos_actors::{
    io::{Data, Read, Write},
    Update,
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
impl Update for Filter {
    fn update(&mut self) {
        self.data += 0.05
            * (2. * std::f64::consts::PI * self.step as f64 * (1e3f64 * 2e-2).recip()).sin()
            + self.noise.sample(&mut rand::thread_rng());
        self.step += 1;
    }
}
impl Read<f64, SignalToFilter> for Filter {
    fn read(&mut self, data: Arc<Data<f64, SignalToFilter>>) {
        self.data = **data;
    }
}

pub enum FilterToSink {}
impl Write<f64, FilterToSink> for Filter {
    fn write(&self) -> Option<Arc<Data<f64, FilterToSink>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}

pub enum FilterToSampler {}
impl Write<f64, FilterToSampler> for Filter {
    fn write(&self) -> Option<Arc<Data<f64, FilterToSampler>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}

pub enum FilterToCompensator {}
impl Write<f64, FilterToCompensator> for Filter {
    fn write(&self) -> Option<Arc<Data<f64, FilterToCompensator>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}
