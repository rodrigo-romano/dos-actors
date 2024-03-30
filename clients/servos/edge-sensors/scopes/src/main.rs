use edge_sensors::{M2S1Tz, M2S1VcAsTz};
use gmt_dos_clients_io::optics::SegmentPiston;
use gmt_dos_clients_scope_client::GridScope;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    loop {
        GridScope::new((1, 2))
            .pin::<SegmentPiston<-9>>((0, 0))?
            .pin::<M2S1VcAsTz>((0, 1))?
            .pin::<M2S1Tz>((0, 1))?
            .show();
    }
    Ok(())
}
