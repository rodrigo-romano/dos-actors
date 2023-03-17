use std::ops::{Mul, Sub};

#[derive(Debug, Default)]
pub struct ScalarIntegrator<T> {
    pub gain: T,
}
impl<T> ScalarIntegrator<T>
where
    T: Default + Copy + Sub<T, Output = T> + Mul<T, Output = T>,
{
    pub fn new(gain: T) -> Self {
        Self { gain }
    }
    pub fn step(&self, u: T, y: T) -> T {
        y - self.gain * u
    }
}
