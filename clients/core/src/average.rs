use super::{Data, Read, UniqueIdentifier, Update, Write};
use std::{
    fmt::Debug,
    marker::PhantomData,
    mem,
    ops::{AddAssign, DivAssign},
};

/// Rate transitionner
#[derive(Debug)]
pub struct Average<
    T,
    U: UniqueIdentifier<DataType = Vec<T>>,
    V: UniqueIdentifier<DataType = Vec<T>> = U,
> {
    data: Vec<T>,
    count: u32,
    n_write: usize,
    input: PhantomData<U>,
    output: PhantomData<V>,
}
impl<T, U, V> Average<T, U, V>
where
    T: Default + Clone,
    U: UniqueIdentifier<DataType = Vec<T>>,
    V: UniqueIdentifier<DataType = Vec<T>>,
{
    /// Creates a new sampler with initial condition
    pub fn new(n_data: usize) -> Self {
        Self {
            data: vec![T::default(); n_data],
            count: 0,
            n_write: 0,
            input: PhantomData,
            output: PhantomData,
        }
    }
}
impl<T, U: UniqueIdentifier<DataType = Vec<T>>, V: UniqueIdentifier<DataType = Vec<T>>> Update
    for Average<T, U, V>
where
    T: Send + Sync,
{
}
impl<U, T, V> Read<U> for Average<T, U, V>
where
    U: UniqueIdentifier<DataType = Vec<T>>,
    T: Copy + AddAssign + Send + Sync,
    V: UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Data<U>) {
        self.data
            .iter_mut()
            .zip(&**data)
            .for_each(|(u, &x)| *u += x);
        self.count += 1;
    }
}
impl<T, U, V> Write<V> for Average<T, U, V>
where
    T: Copy + DivAssign + TryFrom<u32> + Default + Debug + Send + Sync,
    U: UniqueIdentifier<DataType = Vec<T>>,
    V: UniqueIdentifier<DataType = Vec<T>>,
{
    fn write(&mut self) -> Option<Data<V>> {
        let n_data = self.data.len();
        if self.count == 0 || self.n_write == 0 {
            self.n_write += 1;
            return Some(Data::new(vec![T::default(); n_data]));
        }
        let Ok(count) = T::try_from(self.count) else {
            return None;
        };
        self.data.iter_mut().for_each(|x| *x /= count);
        self.count = 0;
        self.n_write += 1;
        Some(Data::new(mem::replace(
            &mut self.data,
            vec![T::default(); n_data],
        )))
    }
}
