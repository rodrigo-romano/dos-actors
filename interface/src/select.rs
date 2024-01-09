//! # Vector element selection
//!
//! The module defines the structure [Select]`<U: `[UniqueIdentifier]`, const IDX: usize>` that selects
//! the element at index `IDX` in a vector for a type `U` that implements the trait [UniqueIdentifier]
//! with datatype [UniqueIdentifier::DataType]`=`[Vec]`<T>`

use std::marker::PhantomData;

use crate::{Data, UniqueIdentifier, Update, Write};

/// Marker trait for client
pub trait Selector {}

/**
Vector element selection type

# Example
```
use gmt_dos_actors_clients_interface::{Data, UniqueIdentifier, Update, Write,
    select::{Select, Selector}};
pub enum TTT {}
impl UniqueIdentifier for TTT {
    type DataType = Vec<u32>;
}
pub struct Client {
    pub data: Vec<u32>,
}
impl Update for Client {}
impl Write<TTT> for Client {
    fn write(&mut self) -> Option<Data<TTT>> {
        Some(self.data.clone().into())
    }
}
impl Selector for Client {}
let mut client = Client {
    data: vec![1, 2, 3, 4, 5],
};
let data = <Client as Write<Select<TTT, 3>>>::write(&mut client);
```
*/
pub struct Select<U: UniqueIdentifier, const IDX: usize>(PhantomData<U>);

impl<U: UniqueIdentifier, const IDX: usize> UniqueIdentifier for Select<U, IDX> {
    type DataType = <U as UniqueIdentifier>::DataType;

    const PORT: u32 = <U as UniqueIdentifier>::PORT + 11 * IDX as u32;
}

impl<U: UniqueIdentifier, const IDX: usize> Update for Select<U, IDX> {}

impl<T, C, U, const IDX: usize> Write<Select<U, IDX>> for C
where
    T: Copy,
    C: Write<U> + Selector,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn write(&mut self) -> Option<Data<Select<U, IDX>>> {
        let Some(data) = <C as Write<U>>::write(self) else {
            return None;
        };
        let inner: &[T] = (&data).into();
        let value = vec![inner[IDX]];
        Some(value.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Data, UniqueIdentifier, Update, Write};

    use super::{Select, Selector};

    pub enum TTT {}
    impl UniqueIdentifier for TTT {
        type DataType = Vec<u32>;
    }

    pub struct Client {
        pub data: Vec<u32>,
    }

    impl Update for Client {}

    impl Write<TTT> for Client {
        fn write(&mut self) -> Option<crate::Data<TTT>> {
            Some(self.data.clone().into())
        }
    }

    impl Selector for Client {}

    #[test]
    fn select() {
        let mut client = Client {
            data: vec![1, 2, 3, 4, 5],
        };

        let data = <Client as Write<Select<TTT, 3>>>::write(&mut client);
        dbg!(data);
    }
}
