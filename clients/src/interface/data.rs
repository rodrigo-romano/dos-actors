use std::{fmt, marker::PhantomData, ops::Deref, sync::Arc};

use super::{UniqueIdentifier, Who};

/// Actors I/O data
///
/// `T` is the data primitive type and `U` is the data unique identifier (UID).
/// The data is wrapped into an [Arc] pointer, allowing cheap cloning i.e. multiple instances of
/// the same [Data] object will point to the same data allocation in memory.
pub struct Data<U: UniqueIdentifier>(Arc<<U as UniqueIdentifier>::DataType>, PhantomData<U>);
impl<T, U: UniqueIdentifier<DataType = T>> Deref for Data<U> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

unsafe impl<T: Send, U: UniqueIdentifier<DataType = T>> Send for Data<U> {}
unsafe impl<T: Sync, U: UniqueIdentifier<DataType = T>> Sync for Data<U> {}

impl<T, U: UniqueIdentifier<DataType = T>> Clone for Data<U> {
    /// Makes a clone of the inner `Arc` pointer
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), PhantomData)
    }
}

impl<T, U: UniqueIdentifier<DataType = T>> Data<U> {
    /// Create a new [Data] object
    pub fn new(data: T) -> Self {
        Data(Arc::new(data), PhantomData)
    }
    /// Replaces the UID `U` with `V`
    ///
    /// [Data]`<U>` is consumed in the process.
    pub fn transmute<V: UniqueIdentifier<DataType = T>>(self) -> Data<V> {
        Data(self.0, PhantomData)
    }
    /// Extracts the inner [Arc] pointer
    ///
    /// [Data]`<U>` is consumed in the process.
    pub fn into_arc(self) -> Arc<T> {
        self.0
    }
}
impl<T, U> From<Data<U>> for Vec<T>
where
    T: Clone,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn from(data: Data<U>) -> Self {
        (*data.0).clone()
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
impl<'a, T, U: UniqueIdentifier<DataType = Vec<T>>> From<&'a Data<U>> for &'a [T] {
    fn from(data: &'a Data<U>) -> Self {
        data
    }
}
impl<T, U: UniqueIdentifier<DataType = Vec<T>>> From<Vec<T>> for Data<U> {
    fn from(u: Vec<T>) -> Self {
        Data(Arc::new(u), PhantomData)
    }
}
impl<T, U: UniqueIdentifier<DataType = T>> From<Arc<T>> for Data<U> {
    fn from(u: Arc<T>) -> Self {
        Data(u, PhantomData)
    }
}
impl<T, U: UniqueIdentifier<DataType = T>> From<&Arc<T>> for Data<U> {
    /// Creates a new [Data] object by cloning the [Arc] reference
    fn from(u: &Arc<T>) -> Self {
        Data(Arc::clone(u), PhantomData)
    }
}

impl<T, U, V> From<&Data<V>> for Data<U>
where
    U: UniqueIdentifier<DataType = T>,
    V: UniqueIdentifier<DataType = T>,
{
    /// Creates a new [Data]`<U>` object by cloning the [Arc] pointer in [Data]`<V>`
    fn from(data: &Data<V>) -> Self {
        Data(Arc::clone(&data.0), PhantomData)
    }
}
impl<U: UniqueIdentifier> Who<U> for Data<U> {}
impl<T, U> fmt::Debug for Data<U>
where
    T: fmt::Debug,
    U: UniqueIdentifier<DataType = T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Data").field(&self.0).field(&self.1).finish()
    }
}
impl<T: Default, U: UniqueIdentifier<DataType = T>> Default for Data<U> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}
impl<T, U> PartialEq for Data<U>
where
    T: PartialEq,
    U: UniqueIdentifier<DataType = T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
use std::hash::Hash;
impl<T: Hash, U: UniqueIdentifier<DataType = T>> Hash for Data<U> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}
