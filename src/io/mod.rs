/*!
# Actor inputs and outputs implementation module

[Actor]s communicate using channels, one input of an [actor] send data through
either a [bounded] or an [unbounded] channel to an output of another actor.
The data that moves through a channel is encapsulated into a [Data] structure.

Each input and output has a reference to the [Actor] client that reads data from
 the input and write data to the output only if the client implements the [Read]
and [Write] traits.

[Actor]: crate::Actor
[bounded]: https://docs.rs/flume/latest/flume/fn.bounded
[unbounded]: https://docs.rs/flume/latest/flume/fn.unbounded
*/

use crate::Who;
use std::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

mod input;
pub(crate) use input::{Input, InputObject};
mod output;
pub(crate) use output::{Output, OutputObject};

pub(crate) type Assoc<U> = <U as UniqueIdentifier>::Data;

/// Defines the data type associated with unique identifier data type
pub trait UniqueIdentifier: Send + Sync {
    type Data;
}

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
impl<T, U: UniqueIdentifier<Data = T>> Data<U> {
    /// Create a new [Data] object
    pub fn new(data: T) -> Self {
        Data(data, PhantomData)
    }
    pub fn into<V: UniqueIdentifier<Data = T>>(self) -> Data<V> {
        Data::new(self.0)
    }
}
impl<T, U: UniqueIdentifier<Data = Vec<T>>> From<Data<U>> for Vec<T>
where
    T: Default,
{
    fn from(mut data: Data<U>) -> Self {
        std::mem::take(&mut data)
    }
}
impl<T, U: UniqueIdentifier<Data = Vec<T>>> From<&Data<U>> for Vec<T>
where
    T: Clone,
{
    fn from(data: &Data<U>) -> Self {
        data.to_vec()
    }
}
impl<T, U: UniqueIdentifier<Data = Vec<T>>> From<&mut Data<U>> for Vec<T>
where
    T: Clone,
{
    fn from(data: &mut Data<U>) -> Self {
        std::mem::take(&mut *data)
    }
}
impl<T, U: UniqueIdentifier<Data = Vec<T>>> From<Vec<T>> for Data<U> {
    /// Returns data UID
    fn from(u: Vec<T>) -> Self {
        Data(u, PhantomData)
    }
}
impl<T, U, V> From<&mut Data<V>> for Data<U>
where
    T: Default,
    U: UniqueIdentifier<Data = T>,
    V: UniqueIdentifier<Data = T>,
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
impl<T: Default, U: UniqueIdentifier<Data = Vec<T>>> Default for Data<U> {
    fn default() -> Self {
        Data::new(Default::default())
    }
}

pub(crate) type S<U> = Arc<Data<U>>;

/// Client input data reader interface
pub trait Read<U: UniqueIdentifier> {
    /// Read data from an input
    fn read(&mut self, data: Arc<Data<U>>);
}
/// Client output data writer interface
pub trait Write<U: UniqueIdentifier> {
    fn write(&mut self) -> Option<Arc<Data<U>>>;
}

#[cfg(test)]
mod tests {
    use crate as uid;
    use uid_derive::UID;

    #[derive(UID)]
    #[uid(data = "u8")]
    pub enum A {}

    #[test]
    fn impl_uid() {
        enum U {}
        impl uid::UniqueIdentifier for U {
            type Data = f64;
        }
        let _: <U as uid::UniqueIdentifier>::Data = 1f64;
    }

    #[test]
    fn derive() {
        #[derive(UID)]
        enum U {}
        let _: <U as uid::UniqueIdentifier>::Data = vec![1f64];
    }

    #[test]
    fn derive_uid() {
        #[derive(UID)]
        #[uid(data = "Vec<f32>")]
        enum U {}
        let _: <U as uid::UniqueIdentifier>::Data = vec![1f32];
    }
}
