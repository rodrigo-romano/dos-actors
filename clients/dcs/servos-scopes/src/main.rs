use gmt_dos_clients_io::mount::{AverageMountEncoders, MountSetPoint};
use gmt_dos_clients_scope_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    gmt_dos_clients_scope_client::Scope::new()
        .signal::<MountSetPoint>()?
        .signal::<AverageMountEncoders>()?
        .show();
    Ok(())
}
