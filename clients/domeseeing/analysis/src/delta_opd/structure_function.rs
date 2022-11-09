use serde::{Deserialize, Serialize};
use statrs::function::gamma::gamma;
use std::{f64::consts::PI, ops::Deref};

#[derive(Debug, Serialize, Deserialize)]
pub struct StructureFunction {
    pub(crate) baseline: f64,
    value: f64,
}
impl StructureFunction {
    pub fn new(baseline: f64, value: f64) -> Self {
        Self { baseline, value }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StructureFunctionSample {
    sf: Vec<StructureFunction>,
    pub(crate) power_law_fit: Option<((f64, f64), f64)>,
}

impl Deref for StructureFunctionSample {
    type Target = Vec<StructureFunction>;

    fn deref(&self) -> &Self::Target {
        &self.sf
    }
}
impl StructureFunctionSample {
    pub fn new(sf: Vec<StructureFunction>) -> Self {
        Self {
            sf,
            power_law_fit: None,
        }
    }
    pub fn update_fit(&mut self, power_law_fit: ((f64, f64), f64)) -> &mut Self {
        self.power_law_fit = Some(power_law_fit);
        self
    }
    pub fn power_law_fit(&mut self) -> ((f64, f64), f64) {
        let (x, y): (Vec<_>, Vec<_>) = self
            .iter()
            .map(
                |StructureFunction {
                     baseline: r,
                     value: sf,
                 }| ((std::f64::consts::PI * r).ln(), sf.ln()),
            )
            .unzip();
        let fit = crate::polyfit(&x, &y, 1).unwrap();
        let n = x.len() as f64;
        let residue = (x
            .into_iter()
            .zip(y.into_iter())
            .map(|(x, y)| y - (x * fit[1] + fit[0]))
            .map(|x| x * x)
            .sum::<f64>()
            / n)
            .sqrt();
        let c = fit[1] / 2. + 1.;
        let a = -1. * fit[0].exp() * gamma(c).powi(2) * (PI * c).sin() / (2. * PI.powi(2));
        self.power_law_fit = Some(((a, c), residue));
        ((a, c), residue)
    }
}
pub struct StructureFunctionSubSample<'a> {
    pub(crate) sf: Vec<&'a StructureFunction>,
}
impl<'a> Deref for StructureFunctionSubSample<'a> {
    type Target = Vec<&'a StructureFunction>;

    fn deref(&self) -> &Self::Target {
        &self.sf
    }
}
impl<'a> StructureFunctionSubSample<'a> {
    pub fn power_law_fit(&mut self) -> ((f64, f64), f64) {
        let (x, y): (Vec<_>, Vec<_>) = self
            .iter()
            .map(
                |StructureFunction {
                     baseline: r,
                     value: sf,
                 }| ((std::f64::consts::PI * r).ln(), sf.ln()),
            )
            .unzip();
        let fit = crate::polyfit(&x, &y, 1).unwrap();
        let n = x.len() as f64;
        let residue = (x
            .into_iter()
            .zip(y.into_iter())
            .map(|(x, y)| y - (x * fit[1] + fit[0]))
            .map(|x| x * x)
            .sum::<f64>()
            / n)
            .sqrt();
        let c = fit[1] / 2. + 1.;
        let a = -1. * fit[0].exp() * gamma(c).powi(2) * (PI * c).sin() / (2. * PI.powi(2));
        ((a, c), residue)
    }
}
