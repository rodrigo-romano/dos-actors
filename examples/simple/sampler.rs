use crate::FilterToSampler;
use dos_actors::{
    io::{Consuming, Data, Producing},
    Updating,
};
use std::sync::Arc;

#[derive(Default)]
pub struct Sampler(f64);
impl Updating for Sampler {}
impl Consuming<f64, FilterToSampler> for Sampler {
    fn consume(&mut self, data: Arc<Data<f64, FilterToSampler>>) {
        self.0 = **data;
    }
}

pub enum SamplerToSink {}
impl Producing<f64, SamplerToSink> for Sampler {
    fn produce(&self) -> Option<Arc<Data<f64, SamplerToSink>>> {
        Some(Arc::new(Data::new(self.0)))
    }
}
