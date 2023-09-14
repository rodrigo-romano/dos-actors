use super::{Data, Read, UniqueIdentifier, Update, Write};
use std::{marker::PhantomData, sync::Arc};

/// Rate transitionner
#[derive(Debug)]
pub struct Pulse<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T> = U> {
    default: Arc<T>,
    data: Arc<T>,
    width: usize,
    step: usize,
    input: PhantomData<U>,
    output: PhantomData<V>,
}
impl<T, U, V> Pulse<T, U, V>
where
    U: UniqueIdentifier<DataType = T>,
    V: UniqueIdentifier<DataType = T>,
{
    /// Creates a new sampler with initial condition
    pub fn new(width: usize, default: T) -> Self {
        let default = Arc::new(default);
        Self {
            data: Arc::clone(&default),
            default,
            input: PhantomData,
            output: PhantomData,
            width,
            step: 0,
        }
    }
}
impl<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Update
    for Pulse<T, U, V>
{
}
impl<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Read<U>
    for Pulse<T, U, V>
{
    fn read(&mut self, data: Data<U>) {
        self.step = 0;
        self.data = data.into_arc();
    }
}
impl<T, U, V> Write<V> for Pulse<T, U, V>
where
    T: Clone + Default,
    U: UniqueIdentifier<DataType = T>,
    V: UniqueIdentifier<DataType = T>,
    Data<V>: Default,
{
    fn write(&mut self) -> Option<Data<V>> {
        if self.step < self.width {
            self.step += 1;
            Some(Data::<V>::from(&self.data))
        } else {
            self.step += 1;
            Some(Data::<V>::from(&self.default))
        }
    }
}
