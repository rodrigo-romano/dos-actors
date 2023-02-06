use gmt_dos_actors::{
    io::{Data, Read, Write},
    prelude::*,
    Update,
};
use nanorand::{Rng, WyRand};
use std::sync::Arc;
// ANCHOR: client
#[derive(Default)]
struct Client {
    data: i32,
}
// ANCHOR_END: client
// ANCHOR: client_in
#[derive(UID)]
#[uid(data = "i32")]
enum In {}
// ANCHOR_END: client_in
// ANCHOR: client_out
#[derive(UID)]
#[uid(data = "f32")]
enum Out {}
// ANCHOR_END: client_out
// ANCHOR: client_io_update
impl Update for Client {}
// ANCHOR_END: client_io_update
// ANCHOR: client_io_read
impl Read<In> for Client {
    fn read(&mut self, data: Arc<Data<In>>) {
        self.data = **data;
    }
}
// ANCHOR_END: client_io_read
// ANCHOR: client_io_write
impl Write<Out> for Client {
    fn write(&mut self) -> Option<Arc<Data<Out>>> {
        Some(Arc::new(Data::new(self.data as f32 * std::f32::consts::E)))
    }
}
// ANCHOR_END: client_io_write

// ANCHOR: rand_gen
struct RandGen {
    data: Vec<i32>,
}
impl RandGen {
    pub fn new(n_sample: usize) -> Self {
        let mut data = vec![0i32; n_sample];
        let mut rng = WyRand::new();
        rng.fill(&mut data);
        Self { data }
    }
}
impl Update for RandGen {}
impl Write<In> for RandGen {
    fn write(&mut self) -> Option<Arc<Data<In>>> {
        self.data.pop().map(|val| Arc::new(Data::new(val)))
    }
}
// ANCHOR_END: rand_gen
// ANCHOR: data_log
#[derive(Default)]
struct DataLogger {
    data: Vec<f32>,
}
impl Update for DataLogger {}
impl Read<Out> for DataLogger {
    fn read(&mut self, data: Arc<Data<Out>>) {
        self.data.push(**data);
    }
}
// ANCHOR_END: data_log

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ANCHOR: env_logger
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();
    // ANCHOR_END: env_logger
    // ANCHOR: actors
    let mut source = Initiator::<_>::from(RandGen::new(1_000_000));
    //// ANCHOR: client_to_actor
    let mut filter = Actor::<_>::from(Client::default());
    //// ANCHOR_END: client_to_actor
    let mut log = Terminator::<_>::from(DataLogger::default());
    // ANCHOR_END: actors
    // ANCHOR: actors_network
    source.add_output().build::<In>().into_input(&mut filter);
    filter
        .add_output()
        .unbounded()
        .build::<Out>()
        .into_input(&mut log);
    // ANCHOR_END: actors_network
    // ANCHOR: model
    model!(source, filter, log)
        .flowchart()
        .check()?
        .run()
        .await?;
    // ANCHOR_END: model
    Ok(())
}
