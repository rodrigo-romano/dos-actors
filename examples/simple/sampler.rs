use crate::FilterToSampler;
use dos_actors::{
    io::{Data, Read, Write},
    UniqueIdentifier, Update,
};
use std::sync::Arc;

#[derive(Default)]
pub struct Sampler(f64);
impl Update for Sampler {}
impl Read<f64, FilterToSampler> for Sampler {
    fn read(&mut self, data: Arc<Data<FilterToSampler>>) {
        self.0 = **data;
    }
}

pub enum SamplerToSink {}
impl UniqueIdentifier for SamplerToSink {
    type Data = f64;
}
impl Write<f64, SamplerToSink> for Sampler {
    fn write(&mut self) -> Option<Arc<Data<SamplerToSink>>> {
        Some(Arc::new(Data::new(self.0)))
    }
}
