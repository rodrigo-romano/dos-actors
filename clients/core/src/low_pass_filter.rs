use std::{
    ops::{Add, AddAssign, Mul, Sub},
    sync::Arc,
};

use interface::{Data, Read, UniqueIdentifier, Update, Write};

pub struct LowPassFilter<T> {
    u: Arc<Vec<T>>,
    y: Vec<T>,
    g: T,
}

impl<T: Default + Clone> LowPassFilter<T> {
    pub fn new(n: usize, g: T) -> Self {
        Self {
            u: Arc::new(vec![T::default(); n]),
            y: vec![T::default(); n],
            g,
        }
    }
}

impl<T> Update for LowPassFilter<T>
where
    T: Send + Sync + Sub<Output = T> + Add<Output = T> + Mul<Output = T> + AddAssign + Copy,
{
    fn update(&mut self) {
        let g = self.g;
        self.u
            .iter()
            .zip(self.y.iter_mut())
            .map(|(u, y)| (*u - *y, y))
            .for_each(|(e, y)| {
                *y += g * e;
            });
    }
}

impl<T, U> Read<U> for LowPassFilter<T>
where
    T: Send + Sync + Sub<Output = T> + Add<Output = T> + Mul<Output = T> + Copy + AddAssign,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Data<U>) {
        self.u = data.into_arc();
    }
}

impl<T, U> Write<U> for LowPassFilter<T>
where
    T: Copy + Send + Sync + Sub<Output = T> + Add<Output = T> + Mul<Output = T> + AddAssign,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn write(&mut self) -> Option<Data<U>> {
        Some(self.y.clone().into())
    }
}
