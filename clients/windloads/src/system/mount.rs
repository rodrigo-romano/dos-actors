use std::ops::{Deref, DerefMut};

use gmt_dos_clients::{Smooth, Weight};
use gmt_dos_clients_io::cfd_wind_loads::CFDMountWindLoads;
use interface::{Read, UniqueIdentifier, Update, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountSmoother(Smooth);

impl MountSmoother {
    pub fn new() -> Self {
        Self(Smooth::new())
    }
}

impl Deref for MountSmoother {
    type Target = Smooth;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MountSmoother {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Update for MountSmoother {
    fn update(&mut self) {
        <Smooth as Update>::update(self);
    }
}

impl Read<Weight> for MountSmoother {
    fn read(&mut self, data: interface::Data<Weight>) {
        <Smooth as Read<Weight>>::read(self, data);
    }
}

impl Read<CFDMountWindLoads> for MountSmoother {
    fn read(&mut self, data: interface::Data<CFDMountWindLoads>) {
        <Smooth as Read<CFDMountWindLoads>>::read(self, data);
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for MountSmoother {
    fn write(&mut self) -> Option<interface::Data<U>> {
        <Smooth as Write<U>>::write(self)
    }
}
