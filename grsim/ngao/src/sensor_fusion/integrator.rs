use std::ops::{Mul, Sub, SubAssign};

use super::Control;

#[derive(Debug, Default, Clone)]
pub struct ScalarIntegrator<T> {
    pub u: T,
    pub y: T,
    pub gain: T,
}
impl<T> ScalarIntegrator<T>
where
    T: ScalarIntegratorTrait<T>,
{
    pub fn new(gain: T) -> Self {
        Self {
            gain,
            ..Default::default()
        }
    }
    pub fn step(&mut self) {
        self.y -= self.gain * self.u;
    }
}

pub trait ScalarIntegratorTrait<T>:
    Default + Copy + Sub<T, Output = T> + SubAssign<T> + Mul<T, Output = T> + num_traits::float::Float
{
}

impl<T> ScalarIntegratorTrait<T> for T where
    T: Default
        + Copy
        + Sub<T, Output = T>
        + SubAssign<T>
        + Mul<T, Output = T>
        + num_traits::float::Float
{
}

/* #[derive(Debug, Default, Clone)]
pub struct ScalarIntegratorWithState<T> {
    pub u1: T,
    pub u2: T,
    pub y: T,
    pub state: T,
    pub gain: T,
}
impl<T> ScalarIntegratorWithState<T>
where
    T: ScalarIntegratorTrait<T>,
{
    pub fn new(gain: T) -> Self {
        Self {
            gain,
            ..Default::default()
        }
    }
    pub fn step(&mut self) {
        self.y -= self.gain * self.u;
        // self.state += self.u2;
        // self.y += self.state - self.gain * self.u1;
    }
}

impl Control for ScalarIntegratorWithState<f64> {
    fn get_u(&self) -> f64 {
        todo!()
    }

    fn get_y(&self) -> f64 {
        todo!()
    }

    fn set_u(&mut self, value: f64) {
        todo!()
    }

    fn set_y(&mut self, value: f64) {
        todo!()
    }

    fn step(&mut self) {
        todo!()
    }
} */
