use crate::FilterToSampler;
use dos_actors::{
    io::{Data, Read, Write},
    Update,
};
use std::sync::Arc;
use uid_derive::UID;

#[derive(Default)]
pub struct Sampler(f64);
impl Update for Sampler {}
impl Read<FilterToSampler> for Sampler {
    fn read(&mut self, data: Arc<Data<FilterToSampler>>) {
        self.0 = **data;
    }
}

#[derive(UID)]
#[uid(data = "f64")]
pub enum SamplerToSink {}
impl Write<SamplerToSink> for Sampler {
    fn write(&mut self) -> Option<Arc<Data<SamplerToSink>>> {
        Some(Arc::new(Data::new(self.0)))
    }
}
