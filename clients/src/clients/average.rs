use super::{Data, Read, UniqueIdentifier, Update, Write};
use std::{
    fmt::Debug,
    marker::PhantomData,
    mem,
    ops::{AddAssign, DivAssign},
    sync::Arc,
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
            input: PhantomData,
            output: PhantomData,
        }
    }
}
impl<T, U: UniqueIdentifier<DataType = Vec<T>>, V: UniqueIdentifier<DataType = Vec<T>>> Update
    for Average<T, U, V>
{
}
impl<U, T, V> Read<U> for Average<T, U, V>
where
    U: UniqueIdentifier<DataType = Vec<T>>,
    T: Copy + AddAssign,
    V: UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Arc<Data<U>>) {
        self.data
            .iter_mut()
            .zip(&**data)
            .for_each(|(u, &x)| *u += x);
        self.count += 1;
    }
}
impl<T, U, V> Write<V> for Average<T, U, V>
where
    T: Copy + DivAssign + TryFrom<u32> + Default + Debug,
    U: UniqueIdentifier<DataType = Vec<T>>,
    V: UniqueIdentifier<DataType = Vec<T>>,
{
    fn write(&mut self) -> Option<Arc<Data<V>>> {
        if self.count == 0 {
            return None;
        }
        let Ok(count) = T::try_from(self.count) else {
                return None;
        };
        let n_data = self.data.len();
        self.data.iter_mut().for_each(|x| *x /= count);
        self.count = 0;
        Some(Arc::new(Data::new(mem::replace(
            &mut self.data,
            vec![T::default(); n_data],
        ))))
    }
}
