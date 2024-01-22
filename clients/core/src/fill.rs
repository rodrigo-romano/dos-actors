use std::{ops::Neg, sync::Arc};

use interface::{Data, Read, UniqueIdentifier, Update, Write};

// #[derive(Default)]
pub struct Fill<T> {
    value: T,
    data: Arc<Vec<T>>,
    n: usize,
}

impl<T> Fill<T> {
    pub fn new(value: T, n: usize) -> Self {
        Fill {
            value,
            data: Arc::new(vec![]),
            n,
        }
    }
}

impl<T: Send + Sync> Update for Fill<T> {}

impl<T, U> Read<U> for Fill<T>
where
    T: Send + Sync,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn read(&mut self, data: Data<U>) {
        self.data = data.into_arc();
    }
}

impl<T, U> Write<U> for Fill<T>
where
    T: Send + Sync + Copy + Neg<Output = T>,
    U: UniqueIdentifier<DataType = Vec<T>>,
{
    fn write(&mut self) -> Option<Data<U>> {
        let n_data = self.data.len();
        assert!(self.n >= n_data);
        let data: Vec<T> = self
            .data
            .iter()
            .map(|&x| -x)
            .chain(vec![self.value; self.n - n_data].into_iter())
            .collect();
        Some(data.into())
    }
}
