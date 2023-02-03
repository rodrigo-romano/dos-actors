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

use std::sync::Arc;

mod input;
pub(crate) use input::{Input, InputObject};
mod output;
pub(crate) use output::{Output, OutputObject};
mod data;
pub use data::Data;

pub(crate) type Assoc<U> = <U as UniqueIdentifier>::DataType;

/// Defines the data type associated with unique identifier data type
pub trait UniqueIdentifier: Send + Sync {
    type DataType;
}

pub(crate) type S<U> = Arc<Data<U>>;

/// Actor client state update interface
pub trait Update {
    fn update(&mut self) {}
}
/// Client input data reader interface
pub trait Read<U: UniqueIdentifier> {
    /// Read data from an input
    fn read(&mut self, data: Arc<Data<U>>);
}
/// Client output data writer interface
pub trait Write<U: UniqueIdentifier> {
    fn write(&mut self) -> Option<Arc<Data<U>>>;
}
/// Interface for IO data sizes
pub trait Size<U: UniqueIdentifier> {
    fn len(&self) -> usize;
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
            type DataType = f64;
        }
        let _: <U as uid::UniqueIdentifier>::DataType = 1f64;
    }

    #[test]
    fn derive() {
        #[derive(UID)]
        enum U {}
        let _: <U as uid::UniqueIdentifier>::DataType = vec![1f64];
    }

    #[test]
    fn derive_uid() {
        #[derive(UID)]
        #[uid(data = "Vec<f32>")]
        enum U {}
        let _: <U as uid::UniqueIdentifier>::DataType = vec![1f32];
    }
}
