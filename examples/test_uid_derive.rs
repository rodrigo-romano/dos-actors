use gmt_dos_actors::{
    io::{Data, Size, UniqueIdentifier, Write},
    UID,
};
use std::sync::Arc;

/// Original
#[derive(UID)]
#[uid(data = "u8")]
pub enum A {}
pub struct Client {}
impl Write<A> for Client {
    fn write(&mut self) -> Option<Arc<Data<A>>> {
        Some(Arc::new(Data::new(10u8)))
    }
}
impl Size<A> for Client {
    fn len(&self) -> usize {
        123
    }
}
#[derive(UID)]
#[alias(name = "A", client = "Client", traits = "Write,Size")]
pub enum B {}

#[derive(UID)]
#[uid(data = "(f64,f32,usize)")]
pub enum TT {}

fn main() {
    let _: <A as UniqueIdentifier>::DataType = 1u8;
    let _: <B as UniqueIdentifier>::DataType = 2u8;

    let mut client = Client {};
    println!(
        "Client Write<B>: {:?}",
        <Client as Write<B>>::write(&mut client)
    );
    println!(
        "Client Size<B>: {:?}",
        <Client as Size<B>>::len(&mut client)
    );
}
