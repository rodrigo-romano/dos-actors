use std::ops::{Deref, DerefMut};

use gmt_dos_clients::{Smooth, Weight};
use gmt_dos_clients_io::cfd_wind_loads::CFDM1WindLoads;
use interface::{Read, UniqueIdentifier, Update, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M1Smoother(Smooth);

impl M1Smoother {
    pub fn new() -> Self {
        Self(Smooth::new())
    }
}

impl Deref for M1Smoother {
    type Target = Smooth;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for M1Smoother {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Update for M1Smoother {
    fn update(&mut self) {
        <Smooth as Update>::update(self);
    }
}

impl Read<Weight> for M1Smoother {
    fn read(&mut self, data: interface::Data<Weight>) {
        <Smooth as Read<Weight>>::read(self, data);
    }
}

impl Read<CFDM1WindLoads> for M1Smoother {
    fn read(&mut self, data: interface::Data<CFDM1WindLoads>) {
        <Smooth as Read<CFDM1WindLoads>>::read(self, data);
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for M1Smoother {
    fn write(&mut self) -> Option<interface::Data<U>> {
        <Smooth as Write<U>>::write(self)
    }
}
