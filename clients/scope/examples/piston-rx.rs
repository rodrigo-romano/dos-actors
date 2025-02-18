use gmt_dos_clients_io::optics::WfeRms;
use gmt_dos_clients_scope::client::Scope;

#[tokio::main]
async fn main() {
    Scope::new().signal::<WfeRms>().unwrap().show();
}
