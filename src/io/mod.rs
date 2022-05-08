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

use crate::{UniqueIdentifier, Who};
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
}
impl<T, U: UniqueIdentifier<Data = Vec<T>>> From<&Data<U>> for Vec<T>
where
    T: Clone,
{
    fn from(data: &Data<U>) -> Self {
        data.to_vec()
    }
}
impl<T, U: UniqueIdentifier<Data = Vec<T>>> From<Vec<T>> for Data<U> {
    /// Returns data UID
    fn from(u: Vec<T>) -> Self {
        Data(u, PhantomData)
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
pub trait Read<T, U: UniqueIdentifier<Data = T>> {
    /// Read data from an input
    fn read(&mut self, data: Arc<Data<U>>);
}
/// Client output data writer interface
pub trait Write<T, U: UniqueIdentifier<Data = T>> {
    fn write(&mut self) -> Option<Arc<Data<U>>>;
}
