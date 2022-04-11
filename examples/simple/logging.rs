use crate::{DifferentiatorToIntegrator, FilterToSink, SamplerToSink, SignalToFilter};
use dos_actors::{
    io::{Data, Read},
    Update,
};
use std::{ops::Deref, sync::Arc};

#[derive(Default)]
pub struct Logging(Vec<f64>);
impl Deref for Logging {
    type Target = Vec<f64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Update for Logging {}
impl Read<f64, SignalToFilter> for Logging {
    fn read(&mut self, data: Arc<Data<f64, SignalToFilter>>) {
        self.0.push(**data);
    }
}
impl Read<f64, FilterToSink> for Logging {
    fn read(&mut self, data: Arc<Data<f64, FilterToSink>>) {
        self.0.push(**data);
    }
}
impl Read<f64, SamplerToSink> for Logging {
    fn read(&mut self, data: Arc<Data<f64, SamplerToSink>>) {
        self.0.push(**data);
    }
}
impl Read<f64, DifferentiatorToIntegrator> for Logging {
    fn read(&mut self, data: Arc<Data<f64, DifferentiatorToIntegrator>>) {
        self.0.push(**data);
    }
}
