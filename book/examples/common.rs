use gmt_dos_actors::{
    io::{Data, Read, Write},
    Update, UID,
};
use std::sync::Arc;

#[derive(UID)]
pub enum U {}
#[derive(UID)]
pub enum Y {}
#[derive(UID)]
pub enum E {}
#[derive(UID)]
pub enum A {}

pub struct Sum {
    left: Arc<Data<U>>,
    right: Arc<Data<Y>>,
}
impl Default for Sum {
    fn default() -> Self {
        Self {
            left: Arc::new(Data::new(vec![])),
            right: Arc::new(Data::new(vec![])),
        }
    }
}
impl Update for Sum {}
impl Read<U> for Sum {
    fn read(&mut self, data: Arc<Data<U>>) {
        self.left = data.clone();
    }
}
impl Read<Y> for Sum {
    fn read(&mut self, data: Arc<Data<Y>>) {
        self.right = data.clone();
    }
}
impl Write<E> for Sum {
    fn write(&mut self) -> Option<Arc<Data<E>>> {
        Some(Arc::new(Data::new(
            self.left
                .iter()
                .zip(self.right.iter())
                .map(|(l, r)| l + r)
                .collect(),
        )))
    }
}

#[allow(dead_code)]
fn main() {}
