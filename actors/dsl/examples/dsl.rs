use std::{error::Error, ops::Deref};

use gmt_dos_actors_dsl::actorscript;
use gmt_dos_clients::interface::{Data, Read, Update, Write, UID};
use tracing::{info, instrument};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let a = A::new(20);
    let b = B(0);
    let c = C(0);

    actorscript! {
        #[model]
        {
         (1: a<A2B> -> b),
         (10: a<A2C> -> &c[bootstrap]<C2B> -> b)
        }
    };
    model.name("dsl").flowchart().check()?.run().await?;

    Ok(())
}

#[derive(Debug)]
pub struct A {
    n: u8,
    i: u8,
}

impl A {
    pub fn new(n: u8) -> Self {
        Self { n, i: 0 }
    }
}
impl Update for A {}
pub struct B(u8);
impl Update for B {}
pub struct C(u8);
impl Update for C {}

#[derive(UID)]
#[uid(data = "u8")]
enum A2B {}
#[derive(UID)]
#[uid(data = "u8")]
enum A2C {}
#[derive(UID)]
#[uid(data = "u8")]
enum C2B {}

impl Write<A2B> for A {
    fn write(&mut self) -> Option<Data<A2B>> {
        // info!("A write A2B: {}", self.i);
        let data = (self.i < self.n).then(|| Data::new(self.i));
        self.i += 1;
        data
    }
}
impl Read<A2B> for B {
    fn read(&mut self, data: Data<A2B>) {
        self.0 = *data.deref();
        // info!("B read A2B : {}", self.0);
    }
}
impl Write<A2C> for A {
    fn write(&mut self) -> Option<Data<A2C>> {
        info!("A write A2C: {}", self.i);
        (self.i < self.n).then(|| Data::new(self.i))
    }
}
impl Read<A2C> for C {
    fn read(&mut self, data: Data<A2C>) {
        self.0 = *data.deref();
        info!("C read A2C : {}", self.0);
    }
}
impl Write<C2B> for C {
    fn write(&mut self) -> Option<Data<C2B>> {
        info!("C write C2B: {}", self.0);
        Some(Data::new(self.0))
    }
}
impl Read<C2B> for B {
    fn read(&mut self, data: Data<C2B>) {
        self.0 = *data.deref();
        info!("B read C2B : {}", self.0);
    }
}
