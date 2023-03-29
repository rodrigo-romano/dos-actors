use super::{Data, Read, UniqueIdentifier, Update, Write};
use std::marker::PhantomData;

/// Rate transitionner
#[derive(Debug)]
pub struct Pulse<T, U: UniqueIdentifier<DataType = T>, V: UniqueIdentifier<DataType = T> = U> {
    input: Data<U>,
    width: usize,
    step: usize,
    output: PhantomData<V>,
}
impl<T, U, V> Pulse<T, U, V>
where
    U: UniqueIdentifier<DataType = T>,
    V: UniqueIdentifier<DataType = T>,
    T: Default,
{
    /// Creates a new sampler with initial condition
    pub fn new(width: usize) -> Self {
        Self {
            input: Data::new(T::default()),
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
        self.input = data;
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
            Some(self.input.transmute())
        } else {
            self.step += 1;
            Some(Data::new(Default::default()))
        }
    }
}
