/*!
# GMT Finite Element Model client

The module implements the client interface for the [GMT FEM Rust API](https://docs.rs/gmt-fem)

*/

#[doc(hidden)]
pub mod prelude {
    pub use crate::{DiscreteModalSolver, Get, Set, Solver};
    pub use interface::{Data, Read, Size, UniqueIdentifier, Update, Write};
    pub mod fem_io {
        pub use gmt_dos_clients_io::gmt_fem::inputs::*;
        pub use gmt_dos_clients_io::gmt_fem::outputs::*;
    }
    pub use std::sync::Arc;
}

use interface::Units;
use prelude::*;

#[cfg(fem)]
pub mod cfd;
#[cfg(fem)]
pub mod m1;
#[cfg(fem)]
pub mod m2;
#[cfg(fem)]
pub mod mount;

impl<S> Units for DiscreteModalSolver<S> where S: Solver + Default {}

impl<S> Update for DiscreteModalSolver<S>
where
    DiscreteModalSolver<S>: Iterator,
    S: Solver + Default + Send + Sync,
{
    fn update(&mut self) {
        log::debug!("update");
        self.next();
    }
}

impl<S, U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for DiscreteModalSolver<S>
where
    DiscreteModalSolver<S>: Iterator,
    Vec<Option<gmt_fem::fem_io::Inputs>>: crate::fem_io::FemIo<U>,
    S: Solver + Default + Send + Sync,
    U: 'static,
{
    fn read(&mut self, data: Data<U>) {
        <DiscreteModalSolver<S> as Set<U>>::set(self, &**data)
    }
}

impl<S, U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for DiscreteModalSolver<S>
where
    DiscreteModalSolver<S>: Iterator,
    Vec<Option<gmt_fem::fem_io::Outputs>>: crate::fem_io::FemIo<U>,
    S: Solver + Default + Send + Sync,
    U: 'static,
{
    fn write(&mut self) -> Option<Data<U>> {
        <DiscreteModalSolver<S> as Get<U>>::get(self).map(|data| Data::new(data))
    }
}
