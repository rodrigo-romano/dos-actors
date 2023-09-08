use std::{error::Error, ops::Deref};

use gmt_dos_actors_dsl::actorscript;
use gmt_dos_clients::interface::{Data, Read, Size, Update, Write, UID};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let a = A::new(20);
    let b = B(1);
    let c = C(1);

    actorscript! {
        #[model(name = demo, state = completed)]
        #[scope(remote)]
         1: a[A2B]$ -> b[C2B]$
         1: a[A2C]$
         10: a[A2C] -> &c[C2B]!$ -> b
    };

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
impl Update for A {
    fn update(&mut self) {
        self.i += 1;
    }
}
pub struct B(u8);
impl Update for B {}
pub struct C(u8);
impl Update for C {}

#[derive(UID)]
#[uid(data = "Vec<u8>")]
enum A2B {}
#[derive(UID)]
#[uid(data = "Vec<u8>")]
enum A2C {}
#[derive(UID)]
#[uid(data = "Vec<u8>")]
enum C2B {}
#[derive(UID)]
#[uid(data = "Vec<u8>")]
enum BB {}

impl Size<A2B> for A {
    fn len(&self) -> usize {
        1
    }
}
impl Size<C2B> for B {
    fn len(&self) -> usize {
        1
    }
}
impl Size<C2B> for C {
    fn len(&self) -> usize {
        1
    }
}
impl Size<A2C> for A {
    fn len(&self) -> usize {
        1
    }
}
impl Size<BB> for B {
    fn len(&self) -> usize {
        1
    }
}

impl Size<BB> for gmt_dos_clients::Sampler<Vec<u8>, BB> {
    fn len(&self) -> usize {
        1
    }
}

impl Write<A2B> for A {
    fn write(&mut self) -> Option<Data<A2B>> {
        // info!("A write A2B: {}", self.i);
        let data = (self.i < self.n).then(|| Data::new(vec![self.i]));

        data
    }
}
impl Read<A2B> for B {
    fn read(&mut self, data: Data<A2B>) {
        self.0 = data.deref()[0];
        // info!("B read A2B : {}", self.0);
    }
}
impl Write<A2C> for A {
    fn write(&mut self) -> Option<Data<A2C>> {
        info!("A write A2C: {}", self.i);
        (self.i < self.n).then(|| Data::new(vec![self.i]))
    }
}
impl Read<A2C> for C {
    fn read(&mut self, data: Data<A2C>) {
        self.0 = data.deref()[0];
        info!("C read A2C : {}", self.0);
    }
}
impl Write<C2B> for C {
    fn write(&mut self) -> Option<Data<C2B>> {
        info!("C write C2B: {}", self.0);
        Some(Data::new(vec![self.0]))
    }
}
impl Read<C2B> for B {
    fn read(&mut self, data: Data<C2B>) {
        self.0 = data.deref()[0];
        info!("B read C2B : {}", self.0);
    }
}
impl Write<C2B> for B {
    fn write(&mut self) -> Option<Data<C2B>> {
        info!("B write C2B: {}", self.0);
        Some(Data::new(vec![self.0]))
    }
}
impl Write<BB> for B {
    fn write(&mut self) -> Option<Data<BB>> {
        info!("B write BB: {}", self.0);
        Some(Data::new(vec![self.0]))
    }
}
