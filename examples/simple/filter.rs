use crate::SignalToFilter;
use dos_actors::{
    io::{Data, Read, Write},
    Update,
};
use rand_distr::{Distribution, Normal};
use std::sync::Arc;
use uid::UniqueIdentifier;
use uid_derive::UID;

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
    fn read(&mut self, data: Arc<Data<SignalToFilter>>) {
        self.data = **data;
    }
}

#[derive(UID)]
#[uid(data = "f64")]
pub enum FilterToSink {}
impl Write<f64, FilterToSink> for Filter {
    fn write(&mut self) -> Option<Arc<Data<FilterToSink>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}

#[derive(UID)]
#[uid(data = "f64")]
pub enum FilterToSampler {}
impl Write<f64, FilterToSampler> for Filter {
    fn write(&mut self) -> Option<Arc<Data<FilterToSampler>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}

#[derive(UID)]
#[uid(data = "f64")]
pub enum FilterToDifferentiator {}
impl Write<f64, FilterToDifferentiator> for Filter {
    fn write(&mut self) -> Option<Arc<Data<FilterToDifferentiator>>> {
        Some(Arc::new(Data::new(self.data)))
    }
}
