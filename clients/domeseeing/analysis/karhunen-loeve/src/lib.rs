use dos_actors::{
    io::{Data, Read},
    prelude::*,
    Update,
};
use std::sync::Arc;

mod karhunenloeve;
pub use karhunenloeve::{
    KarhunenLoeve, KarhunenLoeveCoefficients, KarhunenLoeveResidualCoefficients, ResidualOpd,
};

/// Standard deviation estimate
pub struct Std(Vec<f64>);
impl Std {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}
impl Update for Std {}
impl<U: UniqueIdentifier<Data = Vec<f64>>> Read<U> for Std {
    fn read(&mut self, data: Arc<Data<U>>) {
        let (mut sum_squared, mut sum) =
            data.iter()
                .fold((0f64, 0f64), |(mut sum_squared, mut sum), &o| {
                    sum_squared += o * o;
                    sum += o;
                    (sum_squared, sum)
                });
        let n = data.len() as f64;
        sum_squared /= n;
        sum /= n;
        let std = 1e9 * (sum_squared - sum * sum).sqrt();
        self.0.push(std);
        //println!("STD: {std:4.0}nm")
    }
}
