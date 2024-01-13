//! # Data multiplexer

use std::sync::Arc;

use interface::{Data, Read, UniqueIdentifier, Update, Write};
use std::any::type_name;

/// Multiplexer
///
/// Receive a contiguous vector and chopped it
/// in several vectors which size are given by `slices`
#[derive(Debug, Default)]
pub struct Multiplex<T = f64> {
    data: Arc<Vec<T>>,
    slices: Vec<usize>,
}
impl<T: Default> Multiplex<T> {
    pub fn new(slices: Vec<usize>) -> Self {
        Self {
            slices,
            ..Default::default()
        }
    }
}

impl<T: Send + Sync> Update for Multiplex<T> {}
impl<T, U> Read<U> for Multiplex<T>
where
    U: UniqueIdentifier<DataType = Vec<T>>,
    T: Send + Sync,
{
    fn read(&mut self, data: Data<U>) {
        self.data = data.into_arc();
    }
}
impl<T, U> Write<U> for Multiplex<T>
where
    U: UniqueIdentifier<DataType = Vec<Arc<Vec<T>>>>,
    T: Clone + Send + Sync,
{
    fn write(&mut self) -> Option<Data<U>> {
        let mut mx_data = vec![];
        let data = self.data.as_slice();
        let mut a = 0_usize;
        for s in &self.slices {
            let b = a + *s;
            assert!(b <= data.len(), "{} out of range index", type_name::<U>());
            mx_data.push(Arc::new(data[a..b].to_vec()));
            a = b;
        }
        Some(mx_data.into())
    }
}
