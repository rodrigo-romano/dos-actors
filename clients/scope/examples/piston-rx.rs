use gmt_dos_clients_io::optics::WfeRms;
use gmt_dos_clients_scope::client::Scope;

#[tokio::main]
async fn main() {
    let server_ip = "127.0.0.1";
    let server_port = 5001;
    let client_address = "0.0.0.0:0";

    Scope::new(server_ip, client_address)
        .signal::<WfeRms>(server_port)
        .unwrap()
        .show();
}
