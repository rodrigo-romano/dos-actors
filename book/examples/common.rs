use gmt_dos_clients::interface::{Data, Read, Update, Write, UID};

#[derive(UID)]
pub enum U {}
#[derive(UID)]
pub enum Y {}
#[derive(UID)]
pub enum E {}
#[derive(UID)]
pub enum A {}

pub struct Sum {
    left: Data<U>,
    right: Data<Y>,
}
impl Default for Sum {
    fn default() -> Self {
        Self {
            left: Data::new(vec![]),
            right: Data::new(vec![]),
        }
    }
}
impl Update for Sum {}
impl Read<U> for Sum {
    fn read(&mut self, data: Data<U>) {
        self.left = data.clone();
    }
}
impl Read<Y> for Sum {
    fn read(&mut self, data: Data<Y>) {
        self.right = data.clone();
    }
}
impl Write<E> for Sum {
    fn write(&mut self) -> Option<Data<E>> {
        Some(Data::new(
            self.left
                .iter()
                .zip(self.right.iter())
                .map(|(l, r)| l + r)
                .collect(),
        ))
    }
}

#[allow(dead_code)]
fn main() {}
