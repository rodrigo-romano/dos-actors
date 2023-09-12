use dos_uid_derive::UID;

enum Q<const I: u8> {}

#[derive(UID)]
#[uid(data = Q<I>, port = 9999)]
enum TU {}

fn main() {}
