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

/// input/output data
///
/// `T` is the data primitive type and `U` is the data unique identifier (UID)
pub struct Data<T, U>(T, PhantomData<U>);
impl<T, U> Deref for Data<T, U> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T, U> DerefMut for Data<T, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T, U> Data<T, U> {
    /// Create a new [Data] object
    pub fn new(data: T) -> Self {
        Data(data, PhantomData)
    }
}
impl<T, U> From<&Data<Vec<T>, U>> for Vec<T>
where
    T: Clone,
{
    fn from(data: &Data<Vec<T>, U>) -> Self {
        data.to_vec()
    }
}
impl<T, U> From<Vec<T>> for Data<Vec<T>, U> {
    /// Returns data UID
    fn from(u: Vec<T>) -> Self {
        Data(u, PhantomData)
    }
}
impl<T, U> Who<U> for Data<T, U> {}
impl<T: fmt::Debug, U> fmt::Debug for Data<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&self.who()).field("data", &self.0).finish()
    }
}
impl<T: Default, U> Default for Data<Vec<T>, U> {
    fn default() -> Self {
        Data::new(Default::default())
    }
}

pub(crate) type S<T, U> = Arc<Data<T, U>>;

/// Client input data reader interface
pub trait Read<T, U> {
    /// Read data from an input
    fn read(&mut self, data: Arc<Data<T, U>>);
}
/// Client output data writer interface
pub trait Write<T, U> {
    fn write(&mut self) -> Option<Arc<Data<T, U>>>;
}
