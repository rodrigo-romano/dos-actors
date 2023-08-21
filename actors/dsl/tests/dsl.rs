use std::error::Error;

use gmt_dos_actors_dsl::actorscript;
use gmt_dos_clients::interface::{Data, Read, Update, Write, UID};

pub struct A;
impl Update for A {}
pub struct B;
impl Update for B {}
pub struct C;
impl Update for C {}

#[derive(UID)]
#[uid(data = "u8")]
enum A2B {}

impl Write<A2B> for A {
    fn write(&mut self) -> Option<Data<A2B>> {
        todo!()
    }
}
impl Read<A2B> for B {
    fn read(&mut self, _data: Data<A2B>) {
        todo!()
    }
}
impl Write<A2C> for A {
    fn write(&mut self) -> Option<Data<A2C>> {
        todo!()
    }
}
impl Read<A2C> for C {
    fn read(&mut self, _data: Data<A2C>) {
        todo!()
    }
}
impl Write<C2B> for C {
    fn write(&mut self) -> Option<Data<C2B>> {
        todo!()
    }
}
impl Read<C2B> for B {
    fn read(&mut self, _data: Data<C2B>) {
        todo!()
    }
}

#[derive(UID)]
#[uid(data = "u8")]
enum A2C {}

#[derive(UID)]
#[uid(data = "u8")]
enum C2B {}

fn main() -> Result<(), Box<dyn Error>> {
    let a = A;
    let b = B;
    let c = C;

    actorscript! {
        #[model]
        {
         (1: a<A2B> -> b),
         (10: a<A2C> -> &c<C2B> -> b)
        }
    };

    Ok(())
}
