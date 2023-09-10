use crate::FilterToSampler;
use gmt_dos_clients::interface::{Data, Read, Update, Write, UID};

#[derive(Default)]
pub struct Sampler(f64);
impl Update for Sampler {}
impl Read<FilterToSampler> for Sampler {
    fn read(&mut self, data: Data<FilterToSampler>) {
        self.0 = *data;
    }
}

#[derive(UID)]
#[uid(data = f64)]
pub enum SamplerToSink {}
impl Write<SamplerToSink> for Sampler {
    fn write(&mut self) -> Option<Data<SamplerToSink>> {
        Some(Data::new(self.0))
    }
}
