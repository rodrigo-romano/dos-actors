use crate::{DifferentiatorToIntegrator, FilterToSink, SamplerToSink, SignalToFilter};
use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update};
use std::ops::Deref;

#[derive(Default)]
pub struct Logging(Vec<f64>);
impl Deref for Logging {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Update for Logging {}
impl<T: UniqueIdentifier<DataType = f64>> Read<T> for Logging {
    fn read(&mut self, data: Data<T>) {
        self.0.push(*data);
    }
}
/* impl Read<FilterToSink> for Logging {
    fn read(&mut self, data: Data<FilterToSink>) {
        self.0.push(*data);
    }
}
impl Read<SamplerToSink> for Logging {
    fn read(&mut self, data: Data<SamplerToSink>) {
        self.0.push(*data);
    }
}
impl Read<DifferentiatorToIntegrator> for Logging {
    fn read(&mut self, data: Data<DifferentiatorToIntegrator>) {
        self.0.push(*data);
    }
}
 */
