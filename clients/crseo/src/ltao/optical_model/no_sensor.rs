use crate::ltao::{SensorBuilderProperty, SensorProperty};
use crseo::{Builder, CrseoError, FromBuilder, Propagation, Source};

#[derive(Debug, Default)]
pub struct NoSensor;
impl SensorBuilderProperty for NoSensor {
    fn pupil_sampling(&self) -> usize {
        unimplemented!()
    }
}
impl SensorProperty for NoSensor {
    fn reset(&mut self) {
        unimplemented!()
    }
}

impl Propagation for NoSensor {
    fn propagate(&mut self, _src: &mut Source) {
        unimplemented!()
    }
    fn time_propagate(&mut self, _secs: f64, _src: &mut Source) {
        unimplemented!()
    }
}

impl Builder for NoSensor {
    type Component = NoSensor;
    fn build(self) -> std::result::Result<NoSensor, CrseoError> {
        Ok(NoSensor)
    }
}

impl FromBuilder for NoSensor {
    type ComponentBuilder = NoSensor;
}