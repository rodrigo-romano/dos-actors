use std::env;

use gmt_dos_clients_scope::client::Scope;

const SCOPE_SERVER_IP: &'static str = "127.0.0.1";
const SCOPE_CLIENT_ADDRESS: &'static str = "0.0.0.0:0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_ip = env::var("SCOPE_SERVER_IP").unwrap_or(SCOPE_SERVER_IP.into());

    #[cfg(feature = "wfe-rms_scope")]
    {
        use gmt_dos_clients_io::optics::WfeRms;
        use mechanics::{AsmRefBodyWfeRms, AsmShellWfeRms, M1RbmWfeRms};
        Scope::new(&server_ip, SCOPE_CLIENT_ADDRESS)
            .signal::<WfeRms<-9>>()?
            .signal::<M1RbmWfeRms>()?
            .signal::<AsmShellWfeRms>()?
            .signal::<AsmRefBodyWfeRms>()?
            .show();
    }

    #[cfg(feature = "segment-tip-tilt_scope")]
    {
        use gmt_dos_clients_io::optics::SegmentTipTilt;
        use interface::units::Mas;
        Scope::new(&server_ip, SCOPE_CLIENT_ADDRESS)
            .signal::<Mas<SegmentTipTilt>>()?
            .show();
    }

    #[cfg(feature = "mount_scope")]
    {
        use gmt_dos_clients_io::mount::AverageMountEncoders;
        Scope::new(&server_ip, SCOPE_CLIENT_ADDRESS)
            .signal::<AverageMountEncoders<-6>>()?
            .show();
    }

    #[cfg(feature = "dp21-rss_scope")]
    {
        use gmt_dos_clients_io::optics::SegmentD21PistonRSS;
        use mechanics::{AsmShellSegmentD21PistonRSS, M1RbmSegmentD21PistonRSS};
        Scope::new(&server_ip, SCOPE_CLIENT_ADDRESS)
            .signal::<SegmentD21PistonRSS<-9>>()?
            .signal::<M1RbmSegmentD21PistonRSS>()?
            .signal::<AsmShellSegmentD21PistonRSS>()?
            .show();
    }

    Ok(())
}
