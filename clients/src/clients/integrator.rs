use super::{Data, Read, UniqueIdentifier, Update, Write};
use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Add, AddAssign, Mul, Sub, SubAssign},
};

/// Integral controller
#[derive(Default, Clone)]
pub struct Integrator<U: UniqueIdentifier> {
    gain: U::DataType,
    mem: U::DataType,
    zero: U::DataType,
    skip: usize,
    chunks: Option<usize>,
    uid: PhantomData<U>,
}
impl<T, U> Integrator<U>
where
    T: Default + Clone,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    /// Creates a new integral controller
    pub fn new(n_data: usize) -> Self {
        Self {
            gain: vec![Default::default(); n_data],
            mem: vec![Default::default(); n_data],
            zero: vec![Default::default(); n_data],
            skip: 0,
            chunks: None,
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
    /// Skips the first n data
    ///
    /// Skip is always applied after chunks
    pub fn skip(mut self, n: usize) -> Self {
        self.skip = n;
        self
    }
    /// Process the data by chunks of size n
    pub fn chunks(mut self, n: usize) -> Self {
        self.chunks = Some(n);
        self
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
    /// Sets the gain
    pub fn set_gain(&mut self, gain: T) -> &mut Self {
        self.gain = vec![gain; self.mem.len()];
        self
    }
}
impl<T, U> Update for Integrator<U> where U: UniqueIdentifier<DataType = Vec<T>> {}
impl<T, U> Read<U> for Integrator<U>
where
    T: Copy + Mul<Output = T> + Sub<Output = T> + SubAssign + AddAssign + Debug,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Data<U>) {
        if let Some(chunks) = self.chunks {
            self.mem
                .chunks_mut(chunks)
                .zip(self.gain.chunks(chunks))
                .zip(self.zero.chunks(chunks))
                .zip(data.chunks(chunks - self.skip))
                .for_each(|(((mem, gain), zero), data)| {
                    mem.iter_mut()
                        .zip(gain)
                        .zip(zero)
                        .skip(self.skip)
                        .zip(data)
                        .for_each(|(((x, g), z), u)| *x -= *g * (*u - *z));
                });
        } else {
            self.mem
                .iter_mut()
                .zip(&self.gain)
                .zip(&self.zero)
                .skip(self.skip)
                .zip(&**data)
                .for_each(|(((x, g), z), u)| *x -= *g * (*u - *z));
        }
    }
}
impl<T, V, U> Write<V> for Integrator<U>
where
    T: Copy + Add<Output = T> + Debug,
    V: UniqueIdentifier<DataType = Vec<T>>,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn write(&mut self) -> Option<Data<V>> {
        let y: Vec<T> = self
            .mem
            .iter()
            .zip(&self.zero)
            .map(|(m, z)| *m + *z)
            .collect();
        Some(Data::new(y))
    }
}
