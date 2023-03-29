use std::ops::{Mul, Sub, SubAssign};

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
