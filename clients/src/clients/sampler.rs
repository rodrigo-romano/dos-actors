use super::{Data, Read, UniqueIdentifier, Update, Write};
use std::marker::PhantomData;

/// Rate transitionner
#[derive(Debug)]
pub struct Sampler<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T> = U> {
    input: Data<U>,
    output: PhantomData<V>,
}
impl<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Sampler<T, U, V> {
    /// Creates a new sampler with initial condition
    pub fn new(init: T) -> Self {
        Self {
            input: Data::new(init),
            output: PhantomData,
        }
    }
}
impl<T: Default, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Default
    for Sampler<T, U, V>
{
    fn default() -> Self {
        Self {
            input: Data::new(T::default()),
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
        self.input = data;
    }
}
impl<T: Clone, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Write<V>
    for Sampler<T, U, V>
{
    fn write(&mut self) -> Option<Data<V>> {
        Some((&self.input).into())
    }
}
