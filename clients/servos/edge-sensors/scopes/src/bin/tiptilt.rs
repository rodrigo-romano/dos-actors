#![allow(unreachable_code)]

use gmt_dos_clients_io::optics::TipTilt;
use gmt_dos_clients_scope_client::Scope;
use interface::units::Mas;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("SCOPE_SERVER_IP", "100.21.63.28");
    loop {
        Scope::new()
            .name("Global Tip-Tilt")
            .signal::<Mas<TipTilt>>()?
            .show();
    }
    Ok(())
}
