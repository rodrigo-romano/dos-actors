/*!
# Print

Print the data to the command line

## Examples

Creates a default [Print] client

```
use gmt_dos_clients::print::Print;

let print_data = Print::<f64>::default();
```

*/

use std::sync::Arc;

use interface::{Data, Read, UniqueIdentifier, Update};

/// Print the data to the command line
#[derive(Debug, Default)]
pub struct Print<T> {
    counter: usize,
    data: Option<Vec<Arc<T>>>,
    precision: usize,
}

impl<T: Default> Print<T> {
    /// Creates a new [Print] instance with the given # of digit precision
    pub fn new(precision: usize) -> Self {
        Self {
            precision,
            ..Default::default()
        }
    }
}

impl<T> Update for Print<T>
where
    T: Send + Sync + std::fmt::Debug,
{
    fn update(&mut self) {
        if let Some(data) = self.data.as_ref() {
            println!(
                " #{:>5}: {:+4.precision$?}",
                self.counter,
                data,
                precision = self.precision
            );
            self.counter += 1;
            self.data = None;
        }
    }
}

impl<T, U> Read<U> for Print<T>
where
    T: Send + Sync + std::fmt::Debug,
    U: UniqueIdentifier<DataType = T>,
{
    fn read(&mut self, data: Data<U>) {
        if self.data.is_none() {
            self.data = Some(vec![data.into_arc()])
        } else {
            self.data
                .as_mut()
                .map(|this_data| this_data.push(data.into_arc()));
        }
    }
}
