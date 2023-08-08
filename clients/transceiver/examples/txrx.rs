use gmt_dos_clients::interface::{Read, UniqueIdentifier, UID};
use tracing::info;

#[derive(UID)]
pub enum Sin {}

#[derive(UID)]
pub enum ISin {}

#[derive(UID)]
pub enum Blah {}

#[derive(Debug)]
pub struct Print;

impl gmt_dos_clients::interface::Update for Print {}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for Print {
    fn read(&mut self, data: gmt_dos_clients::interface::Data<U>) {
        info!("sin: {:.3?}", Vec::from(data));
    }
}
