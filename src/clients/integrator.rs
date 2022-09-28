use crate::{
    io::{Data, Read, UniqueIdentifier, Write},
    Update,
};
use std::{
    marker::PhantomData,
    ops::{Add, Mul, Sub, SubAssign},
    sync::Arc,
};

/// Integral controller
#[derive(Default)]
pub struct Integrator<U: UniqueIdentifier> {
    gain: U::Data,
    mem: U::Data,
    zero: U::Data,
    uid: PhantomData<U>,
}
impl<T, U> Integrator<U>
where
    T: Default + Clone,
    U: UniqueIdentifier<Data = Vec<T>>,
{
    /// Creates a new integral controller
    pub fn new(n_data: usize) -> Self {
        Self {
            gain: vec![Default::default(); n_data],
            mem: vec![Default::default(); n_data],
            zero: vec![Default::default(); n_data],
            uid: PhantomData,
        }
    }
    /// Sets a unique gain
    pub fn gain(self, gain: T) -> Self {
        Self {
            gain: vec![gain; self.mem.len()],
            ..self
        }
    }
    /// Sets the gain vector
    pub fn gain_vector(self, gain: Vec<T>) -> Self {
        assert_eq!(
            gain.len(),
            self.mem.len(),
            "gain vector length error: expected {} found {}",
            gain.len(),
            self.mem.len()
        );
        Self { gain, ..self }
    }
    /// Sets the integrator zero point
    pub fn zero(self, zero: Vec<T>) -> Self {
        Self { zero, ..self }
    }
}
impl<T, U> Update for Integrator<U> where U: UniqueIdentifier<Data = Vec<T>> {}
impl<T, U> Read<U> for Integrator<U>
where
    T: Copy + Mul<Output = T> + Sub<Output = T> + SubAssign,
    U: UniqueIdentifier<Data = Vec<T>>,
{
    fn read(&mut self, data: Arc<Data<U>>) {
        self.mem
            .iter_mut()
            .zip(&self.gain)
            .zip(&self.zero)
            .zip(&**data)
            .for_each(|(((x, g), z), u)| *x -= *g * (*u - *z));
    }
}
impl<T, V, U> Write<V> for Integrator<U>
where
    T: Copy + Add<Output = T>,
    V: UniqueIdentifier<Data = Vec<T>>,
    U: UniqueIdentifier<Data = Vec<T>>,
{
    fn write(&mut self) -> Option<Arc<Data<V>>> {
        let y: Vec<T> = self
            .mem
            .iter()
            .zip(&self.zero)
            .map(|(m, z)| *m + *z)
            .collect();
        Some(Arc::new(Data::new(y)))
    }
}
