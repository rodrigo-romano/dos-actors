use nanomsg::{Protocol, Socket};
use rmp_serde::Deserializer;
use serde::Deserialize;
use std::{
    io::{Cursor, Read},
    thread,
    time::Duration,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PvatTrajectoryPoint {
    position: f64,
    velocity: f64,
    acceleration: f64,
    tai: f64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ImMountDemands {
    azimuth_trajectory: Vec<PvatTrajectoryPoint>,
    elevation_trajectory: Vec<PvatTrajectoryPoint>,
    gir_trajectory: Vec<PvatTrajectoryPoint>,
    azimuth_motion_mode: String,
    elevation_motion_mode: String,
    gir_motion_mode: String,
}

fn main() -> anyhow::Result<()> {
    let mut socket = Socket::new(Protocol::Pull)?;
    socket.bind("tcp://127.0.0.1:4242")?;
    let mut buffer = [0u8; 1024];
    loop {
        thread::sleep(Duration::from_millis(1000));
        let count = socket.read(&mut buffer)?;
        println!("Read {} bytes !", count);
        let cur = Cursor::new(&buffer);
        let mut de = Deserializer::new(cur);
        let actual: ImMountDemands = Deserialize::deserialize(&mut de)?;
        println!("{:#?}", actual);
    }
    Ok(())
}
