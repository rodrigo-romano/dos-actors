use std::ops::{Deref, DerefMut};

use gmt_dos_clients::{Smooth, Weight};
use gmt_dos_clients_io::cfd_wind_loads::CFDM2WindLoads;
use interface::{Read, UniqueIdentifier, Update, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M2Smoother(Smooth);

impl M2Smoother {
    pub fn new() -> Self {
        Self(Smooth::new())
    }
}

impl Deref for M2Smoother {
    type Target = Smooth;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for M2Smoother {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Update for M2Smoother {
    fn update(&mut self) {
        <Smooth as Update>::update(self);
    }
}

impl Read<Weight> for M2Smoother {
    fn read(&mut self, data: interface::Data<Weight>) {
        <Smooth as Read<Weight>>::read(self, data);
    }
}

impl Read<CFDM2WindLoads> for M2Smoother {
    fn read(&mut self, data: interface::Data<CFDM2WindLoads>) {
        <Smooth as Read<CFDM2WindLoads>>::read(self, data);
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for M2Smoother {
    fn write(&mut self) -> Option<interface::Data<U>> {
        <Smooth as Write<U>>::write(self)
    }
}
