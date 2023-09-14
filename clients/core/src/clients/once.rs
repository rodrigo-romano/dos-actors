/*!
# Once Per Read

A client that wraps the data it has read into an [Option]
and writes it a  as `Some(...)` once and as `None` until the next read.

## Examples

```
use gmt_dos_clients::{once::Once, interface::{UniqueIdentifier, Read, Write}};
enum O {}
impl UniqueIdentifier for O {
    type DataType = Vec<usize>;
}
enum OO {}
impl UniqueIdentifier for OO {
    type DataType = Option<Vec<usize>>;
}
let mut once = Once::<O>::new();
let data = vec![1,2,3];
<Once<O> as Read<O>>::read(&mut once, data.clone().into());
let written_data = <Once<O> as Write<OO>>::write(&mut once).unwrap();
assert_eq!(*written_data, Some(data));
let written_data = <Once<O> as Write<OO>>::write(&mut once).unwrap();
assert_eq!(*written_data, None);
```

*/

use std::ops::Deref;

use interface::{Data, Read, UniqueIdentifier, Update, Write};

/// Once client
///
/// A client that wraps the data it has read into an [Option]
/// and writes it a  as `Some(...)` once and as `None` until the next read.
pub struct Once<U: UniqueIdentifier> {
    data: Option<U::DataType>,
}
impl<U: UniqueIdentifier> Once<U> {
    pub fn new() -> Self {
        Self { data: None }
    }
}

impl<U: UniqueIdentifier> Default for Once<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: UniqueIdentifier> Update for Once<U> {}

impl<U> Read<U> for Once<U>
where
    U: UniqueIdentifier,
    <U as UniqueIdentifier>::DataType: Clone,
{
    fn read(&mut self, data: Data<U>) {
        self.data = Some(data.deref().clone());
    }
}

impl<U, V> Write<V> for Once<U>
where
    U: UniqueIdentifier,
    V: UniqueIdentifier<DataType = Option<U::DataType>>,
{
    fn write(&mut self) -> Option<Data<V>> {
        Some(Data::new(self.data.take()))
    }
}
