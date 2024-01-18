use gmt_dos_clients_scope::client::Scope;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_ip = "127.0.0.1";
    let client_address = "0.0.0.0:0";
    let n_sample = 250;

    #[cfg(feature = "wfe-rms_scope")]
    {
        use gmt_dos_clients_io::optics::WfeRms;
        use mechanics::{AsmRefBodyWfeRms, AsmShellWfeRms, M1RbmWfeRms};
        Scope::new(server_ip, client_address)
            .signal::<WfeRms<-9>>(5001)?
            .signal::<M1RbmWfeRms>(5002)?
            .signal::<AsmShellWfeRms>(5003)?
            .signal::<AsmRefBodyWfeRms>(5004)?
            .n_sample(n_sample)
            .show();
    }

    #[cfg(feature = "segment-tip-tilt_scope")]
    {
        use gmt_dos_clients_io::optics::SegmentTipTilt;
        use interface::units::Mas;
        Scope::new(server_ip, client_address)
            .signal::<Mas<SegmentTipTilt>>(7001)?
            .n_sample(n_sample)
            .show();
    }

    #[cfg(feature = "mount_scope")]
    {
        use gmt_dos_clients_io::mount::AverageMountEncoders;
        Scope::new(server_ip, client_address)
            .signal::<AverageMountEncoders<-6>>(5005)?
            .n_sample(n_sample)
            .show();
    }

    #[cfg(feature = "dp21-rss_scope")]
    {
        use gmt_dos_clients_io::optics::SegmentD21PistonRSS;
        use mechanics::{AsmShellSegmentD21PistonRSS, M1RbmSegmentD21PistonRSS};
        Scope::new(server_ip, client_address)
            .signal::<SegmentD21PistonRSS<-9>>(6001)?
            .signal::<M1RbmSegmentD21PistonRSS>(6002)?
            .signal::<AsmShellSegmentD21PistonRSS>(6003)?
            .n_sample(n_sample)
            .show();
    }

    Ok(())
}
