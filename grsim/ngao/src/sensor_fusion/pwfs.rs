pub use super::{Control, HdfsOrNot, HdfsOrPwfs, ScalarIntegrator};

// use gmt_ngao_temporal_ctrl::NgaoTemporalCtrl;

// mod requirements;
// pub use requirements::PwfsIntegrator;

mod design;
pub use design::PwfsIntegrator;

pub struct ModesIntegrator<C: Control> {
    pub scint: Vec<C>,
}
impl ModesIntegrator<ScalarIntegrator<f64>> {
    fn single(n_sample: usize, gain: f64) -> Self {
        let scint = vec![ScalarIntegrator::new(gain); n_sample];
        Self { scint }
    }
}
/* impl ModesIntegrator<NgaoTemporalCtrl> {
    fn double(n_sample: usize) -> Self {
        let scint = vec![NgaoTemporalCtrl::new(); n_sample];
        Self { scint }
    }
} */
impl ModesIntegrator<ScalarIntegrator<f64>> {
    fn new(n_sample: usize, gain: f64) -> Self {
        let scint = vec![ScalarIntegrator::new(gain); n_sample];
        Self { scint }
    }
}
