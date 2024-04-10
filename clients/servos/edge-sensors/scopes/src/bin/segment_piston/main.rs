use gmt_dos_clients_io::optics::SegmentPiston;
use gmt_dos_clients_scope_client::Scope;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Scope::new().signal::<SegmentPiston<-9>>()?.show();
    Ok(())
}
