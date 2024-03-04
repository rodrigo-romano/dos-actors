use super::{Data, Read, UniqueIdentifier, Update, Write};
use std::{marker::PhantomData, sync::Arc};

/// Rate transitionner
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct Sampler<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T> = U> {
    data: Arc<T>,
    input: PhantomData<U>,
    output: PhantomData<V>,
}
impl<T: Clone, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T>> Clone
    for Sampler<T, U, V>
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            input: PhantomData,
            output: PhantomData,
        }
    }
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
impl<T, U, V> Update for Sampler<T, U, V>
where
    T: Send + Sync,
    U: UniqueIdentifier<DataType = T>,
    V: UniqueIdentifier<DataType = T>,
{
}
impl<T, U, V> Read<U> for Sampler<T, U, V>
where
    T: Send + Sync,
    U: UniqueIdentifier<DataType = T>,
    V: UniqueIdentifier<DataType = T>,
{
    fn read(&mut self, data: Data<U>) {
        self.data = data.into_arc();
    }
}
impl<T, U, V> Write<V> for Sampler<T, U, V>
where
    T: Clone + Send + Sync,
    U: UniqueIdentifier<DataType = T>,
    V: UniqueIdentifier<DataType = T>,
{
    fn write(&mut self) -> Option<Data<V>> {
        Some(Data::<V>::from(&self.data))
    }
}
