use std::sync::Arc;
use dos_actors::{
    io::{Data, Write},
    Size,
};
use uid::UniqueIdentifier;
use uid_derive::UID;

/// Original
#[derive(UID)]
#[uid(data = "u8")]
pub enum A {}
pub struct Client {}
impl Write<u8, A> for Client {
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

fn main() {
    let _: <A as UniqueIdentifier>::Data = 1u8;
    let _: <B as UniqueIdentifier>::Data = 2u8;

    let mut client = Client {};
    println!(
        "Client Write<B>: {:?}",
        <Client as Write<u8, B>>::write(&mut client)
    );
    println!(
        "Client Size<B>: {:?}",
        <Client as Size<B>>::len(&mut client)
    );
}
