use std::marker::PhantomData;

use dos_uid_derive::UID;
use gmt_dos_clients::interface::{Data, Read, Size, Update, Write};

struct Q<T>(PhantomData<T>);

enum ID {}

#[derive(UID)]
#[uid(data = Q<ID>, port = 9999)]
enum TU {}

struct Client {}

impl Update for Client {}
impl Write<TU> for Client {
    fn write(&mut self) -> Option<gmt_dos_clients::interface::Data<TU>> {
        None
    }
}
impl Read<TU> for Client {
    fn read(&mut self, _data: gmt_dos_clients::interface::Data<TU>) {}
}
impl Size<TU> for Client {
    fn len(&self) -> usize {
        1234
    }
}

#[derive(UID)]
#[uid(data = Q<ID>, port = 999)]
#[alias(name = TU, client = Client, traits = Write, Read, Size)]
enum TUT {}

fn main() {
    let mut client = Client {};
    <Client as Write<TU>>::write(&mut client);
    <Client as Write<TUT>>::write(&mut client);
    let q = Q::<ID>(PhantomData);
    <Client as Read<TU>>::read(&mut client, Data::<TU>::new(q));
    let qq = Q::<ID>(PhantomData);
    <Client as Read<TUT>>::read(&mut client, Data::<TUT>::new(qq));
    println!("TUT size: {}", <Client as Size<TUT>>::len(&client))
}
