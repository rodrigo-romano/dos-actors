use gmt_dos_clients_scope::client::Scope;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "wfe-rms_scope")]
    {
        use gmt_dos_clients_io::optics::WfeRms;
        use mechanics::{AsmRefBodyWfeRms, AsmShellWfeRms, M1RbmWfeRms};
        Scope::new()
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
        Scope::new().signal::<Mas<SegmentTipTilt>>()?.show();
    }

    #[cfg(feature = "mount_scope")]
    {
        use gmt_dos_clients_io::mount::AverageMountEncoders;
        Scope::new().signal::<AverageMountEncoders<-6>>()?.show();
    }

    #[cfg(feature = "dp21-rss_scope")]
    {
        use gmt_dos_clients_io::optics::SegmentD21PistonRSS;
        use mechanics::{AsmShellSegmentD21PistonRSS, M1RbmSegmentD21PistonRSS};
        Scope::new()
            .signal::<SegmentD21PistonRSS<-9>>()?
            .signal::<M1RbmSegmentD21PistonRSS>()?
            .signal::<AsmShellSegmentD21PistonRSS>()?
            .show();
    }

    Ok(())
}
