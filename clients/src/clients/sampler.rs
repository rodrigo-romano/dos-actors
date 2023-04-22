use super::{Data, Read, UniqueIdentifier, Update, Write};
use std::{marker::PhantomData, sync::Arc};

/// Rate transitionner
#[derive(Debug)]
pub struct Sampler<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T> = U> {
    data: Arc<T>,
    input: PhantomData<U>,
    output: PhantomData<V>,
}
impl<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Sampler<T, U, V> {
    /// Creates a new sampler with initial condition
    pub fn new(init: T) -> Self {
        Self {
            data: Arc::new(init),
            input: PhantomData,
            output: PhantomData,
        }
    }
}
impl<T: Default, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Default
    for Sampler<T, U, V>
{
    fn default() -> Self {
        Self {
            data: Default::default(),
            input: PhantomData,
            output: PhantomData,
        }
    }
}
impl<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Update
    for Sampler<T, U, V>
{
}
impl<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Read<U>
    for Sampler<T, U, V>
{
    fn read(&mut self, data: Data<U>) {
        self.data = data.into_arc();
    }
}
impl<T: Clone, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Write<V>
    for Sampler<T, U, V>
{
    fn write(&mut self) -> Option<Data<V>> {
        Some(Data::<V>::from(&self.data))
    }
}
