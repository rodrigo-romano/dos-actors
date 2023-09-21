use std::marker::PhantomData;

use dos_uid_derive::UID;
use interface::{Data, Update, Write};

struct Q<R>(PhantomData<R>);

#[derive(UID)]
#[uid(data = Q<T>, port = 9999)]
struct TU<T: Sync + Send>(PhantomData<T>);

struct Client {}
impl Update for Client {}
impl<T: Sync + Send> Write<TU<T>> for Client {
    fn write(&mut self) -> Option<Data<TU<T>>> {
        None
    }
}

#[derive(UID)]
#[alias(name = TU<T>, client=Client, traits = Write)]
struct TW<T: Sync + Send>(PhantomData<T>);

#[derive(UID)]
enum W<const ID: u8> {}

struct ClientW {}
impl Update for ClientW {}
impl<const ID: u8> Write<W<ID>> for ClientW {
    fn write(&mut self) -> Option<Data<W<ID>>> {
        None
    }
}

#[derive(UID)]
#[alias(name = W<ID>, client=ClientW, traits = Write)]
enum WW<const ID: u8> {}

fn main() {}
