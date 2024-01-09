use interface::{Read, UniqueIdentifier, UID};
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
        info!("{}: {:.3?}", std::any::type_name::<U>(), Vec::from(data));
    }
}
