#![allow(unreachable_code)]

use gmt_dos_clients_io::optics::Wavefront;
use gmt_dos_clients_scope_client::Shot;
use interface::units::NM;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("SCOPE_SERVER_IP", "100.21.63.28");
    loop {
        Shot::new()
            .name("Wavefront")
            .signal::<NM<Wavefront>>()?
            .show();
    }
    Ok(())
}
