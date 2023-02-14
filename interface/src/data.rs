use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{Assoc, UniqueIdentifier, Who};

/// input/output data
///
/// `T` is the data primitive type and `U` is the data unique identifgier (UID)
pub struct Data<U: UniqueIdentifier>(Assoc<U>, PhantomData<U>);
impl<U: UniqueIdentifier> Deref for Data<U> {
    type Target = Assoc<U>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<U: UniqueIdentifier> DerefMut for Data<U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T, U: UniqueIdentifier<DataType = T>> Data<U> {
    /// Create a new [Data] object
    pub fn new(data: T) -> Self {
        Data(data, PhantomData)
    }
    pub fn into<V: UniqueIdentifier<DataType = T>>(self) -> Data<V> {
        Data::new(self.0)
    }
}
impl<T, U: UniqueIdentifier<DataType = Vec<T>>> From<Data<U>> for Vec<T>
where
    T: Default,
{
    fn from(mut data: Data<U>) -> Self {
        std::mem::take(&mut data)
    }
}
impl<T, U: UniqueIdentifier<DataType = Vec<T>>> From<&Data<U>> for Vec<T>
where
    T: Clone,
{
    fn from(data: &Data<U>) -> Self {
        data.to_vec()
    }
}
impl<T, U: UniqueIdentifier<DataType = Vec<T>>> From<&mut Data<U>> for Vec<T>
where
    T: Clone,
{
    fn from(data: &mut Data<U>) -> Self {
        std::mem::take(&mut *data)
    }
}
impl<T, U: UniqueIdentifier<DataType = Vec<T>>> From<Vec<T>> for Data<U> {
    /// Returns data UID
    fn from(u: Vec<T>) -> Self {
        Data(u, PhantomData)
    }
}
impl<T, U, V> From<&mut Data<V>> for Data<U>
where
    T: Default,
    U: UniqueIdentifier<DataType = T>,
    V: UniqueIdentifier<DataType = T>,
{
    /// Returns data UID
    fn from(data: &mut Data<V>) -> Self {
        Data::new(std::mem::take::<T>(&mut *data))
    }
}
impl<U: UniqueIdentifier> Who<U> for Data<U> {}
impl<U> fmt::Debug for Data<U>
where
    U: UniqueIdentifier,
    Assoc<U>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&self.who()).field("data", &self.0).finish()
    }
}
impl<T: Default, U: UniqueIdentifier<DataType = Vec<T>>> Default for Data<U> {
    fn default() -> Self {
        Data::new(Default::default())
    }
}
