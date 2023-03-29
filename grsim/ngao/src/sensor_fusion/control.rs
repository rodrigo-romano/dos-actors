use gmt_ngao_temporal_ctrl::NgaoTemporalCtrl;

use super::ScalarIntegrator;

pub trait Control {
    fn get_u(&self) -> f64;
    fn get_y(&self) -> f64;
    fn set_u(&mut self, value: f64);
    fn set_y(&mut self, value: f64);
    fn step(&mut self);
}
impl Control for ScalarIntegrator<f64> {
    fn get_u(&self) -> f64 {
        self.u
    }

    fn get_y(&self) -> f64 {
        self.y
    }

    fn set_u(&mut self, value: f64) {
        self.u = value;
    }

    fn set_y(&mut self, value: f64) {
        self.y = value;
    }

    fn step(&mut self) {
        self.step();
    }
}
impl Control for NgaoTemporalCtrl {
    fn get_u(&self) -> f64 {
        self.inputs.Delta_m
    }

    fn get_y(&self) -> f64 {
        self.outputs.m
    }

    fn set_u(&mut self, value: f64) {
        self.inputs.Delta_m = value;
    }

    fn set_y(&mut self, value: f64) {
        self.outputs.m = value;
    }

    fn step(&mut self) {
        self.step();
    }
}
