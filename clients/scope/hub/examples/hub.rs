use gmt_dos_clients_io::optics::{SegmentPiston, SegmentWfeRms, TipTilt, WfeRms};
use gmt_dos_clients_scopehub::scopehub;
use interface::units::Arcsec;

#[scopehub]
pub enum MyScopes {
    Scope(WfeRms, SegmentWfeRms),
    Scope(SegmentPiston<-9>),
    Scope(Arcsec<TipTilt>),
}

fn main() {}
