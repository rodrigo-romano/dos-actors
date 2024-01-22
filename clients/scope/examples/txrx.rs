use interface::UID;

#[derive(UID)]
#[uid(port = 5001)]
pub enum Sin {}

#[derive(UID)]
#[uid(port = 5002)]
pub enum Noise {}
