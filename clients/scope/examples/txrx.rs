use gmt_dos_clients::interface::UID;
// use tracing::info;

#[derive(UID)]
pub enum VSin {}

#[derive(UID)]
#[uid(data = "f64")]
pub enum Sin {}

#[derive(UID)]
pub enum VNoise {}

#[derive(UID)]
#[uid(data = "f64")]
pub enum Noise {}

// #[derive(UID)]
// pub enum ISin {}

// #[derive(Debug)]
// pub struct Print;

// impl gmt_dos_clients::interface::Update for Print {}

// impl Read<Sin> for Print {
//     fn read(&mut self, data: gmt_dos_clients::interface::Data<Sin>) {
//         print!("sin: {:.3?}", Vec::from(data));
//     }
// }

// impl Read<ISin> for Print {
//     fn read(&mut self, data: gmt_dos_clients::interface::Data<ISin>) {
//         info!("isin: {:.3?}", Vec::from(data));
//     }
// }