use gmt_dos_clients_io::optics::{SegmentD21PistonRSS, WfeRms};
use gmt_dos_clients_lom::LinearOpticalModel;

use interface::UID;

#[derive(UID)]
#[alias(name = WfeRms<-9>, port = 55991, client = LinearOpticalModel, traits = Write)]
pub enum M1RbmWfeRms {}

#[derive(UID)]
#[alias(name = WfeRms<-9>, port = 55992, client = LinearOpticalModel, traits = Write)]
pub enum AsmShellWfeRms {}

#[derive(UID)]
#[alias(name = WfeRms<-9>, port = 55993, client = LinearOpticalModel, traits = Write)]
pub enum AsmRefBodyWfeRms {}

#[derive(UID)]
#[alias(name = SegmentD21PistonRSS<-9>, port = 55994, client = LinearOpticalModel, traits = Write)]
pub enum M1RbmSegmentD21PistonRSS {}

#[derive(UID)]
#[alias(name = SegmentD21PistonRSS<-9>, port = 55995, client = LinearOpticalModel, traits = Write)]
pub enum AsmShellSegmentD21PistonRSS {}

/* #[derive(Clone)]
pub struct Monitors<const N: usize> {
    wfe_rms: Actor<Scope<WfeRms<-9>>, N, 0>,
    m1_rbm_wfe_rms: Actor<Scope<M1RbmWfeRms>, N, 0>,
    asm_shell_wfe_rms: Actor<Scope<AsmShellWfeRms>, N, 0>,
    asm_ref_body_wfe_rms: Actor<Scope<AsmRefBodyWfeRms>, N, 0>,
}

impl<const N: usize> Display for Monitors<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<const N: usize> System for Monitors<N> {
    fn build(&mut self) -> anyhow::Result<&mut Self> {
        todo!()
    }

    fn plain(&self) -> gmt_dos_actors::actor::PlainActor {
        todo!()
    }
} */
