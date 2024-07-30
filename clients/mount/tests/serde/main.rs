use std::{fs::File, sync::Arc};

use gmt_dos_actors::actor::Actor;
use gmt_dos_clients_mount::Mount;
use interface::filing::Codec;
use tokio::sync::Mutex;

// MOUNT_MODEL=MOUNT_FDR_1kHz cargo test --package gmt_dos-clients_mount --test serde

/// Serializes and deserializes the mount controller client
#[test]
fn main() {
    let mount = Mount::new();
    let serialized = serde_json::to_string_pretty(&mount).unwrap();
    println!("{:#}", serialized);
    let deserialized: Mount = serde_json::from_str(&serialized).unwrap();
    dbg!(&deserialized);
    assert_eq!(
        serde_json::to_string_pretty(&deserialized).unwrap(),
        serialized
    );
}

/// Serializes and deserializes an actor with the mount controller client
#[test]
fn actor() {
    let mount: Actor<Mount, 1, 1> = Actor::new(Arc::new(Mutex::new(Mount::new())));
    let serialized = serde_json::to_string_pretty(&mount).unwrap();
    println!("{:#}", serialized);
    let deserialized: Actor<Mount, 1, 1> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(
        serde_json::to_string_pretty(&deserialized).unwrap(),
        serialized
    );
}

/// Decodes and encodesh the mount controller client to a file
#[test]
fn codec() {
    let mut file = File::create("mount.bin").unwrap();
    let mount = Mount::new();
    mount.encode(&mut file).unwrap();
    let mut file = File::open("mount.bin").unwrap();
    let _mount: Mount = Mount::decode(&mut file).unwrap();
}
