use gmt_dos_clients_mount::Mount;

#[test]
fn main() {
    let mount = Mount::new();
    let serialized = serde_json::to_string_pretty(&mount).unwrap();
    println!("{:#}", serialized);
    let deserialized: Mount = serde_json::from_str(&serialized).unwrap();
    dbg!(deserialized);
}
