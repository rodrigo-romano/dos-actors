//! M1 segment rigid body motions

use super::prelude::*;
use gmt_dos_clients_io::gmt_m1::segment::RBM;

impl<const ID: u8, S: Solver + Default> gmt_dos_clients::interface::Size<RBM<ID>>
    for DiscreteModalSolver<S>
{
    fn len(&self) -> usize {
        42
    }
}

impl<const ID: u8, S: Solver + Default> Write<RBM<ID>> for DiscreteModalSolver<S> {
    fn write(&mut self) -> Option<Data<RBM<ID>>> {
        let a: usize = (ID * 6).into();
        <DiscreteModalSolver<S> as Get<fem_io::OSSM1Lcl>>::get(self)
            .as_ref()
            .map(|data| Data::new((data[a - 6..a]).to_vec()))
    }
}
