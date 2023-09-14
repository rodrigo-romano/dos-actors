use std::error::Error;

use gmt_dos_actors::actorscript;
use gmt_dos_clients_windloads::CfdLoads;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfd_loads = CfdLoads::foh("", 0).m1_segments().build()?;

    actorscript! {
        1: cfd_loads[gmt_dos_clients_io::cfd_wind_loads::CFDM1WindLoads]$~
    }

    Ok(())
}
