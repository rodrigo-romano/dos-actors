use std::ops::{Deref, DerefMut};

use gmt_dos_clients_io::gmt_m2::asm::M2ASMReferenceBodyNodes;
use gmt_dos_clients_lom::LinearOpticalModel;
use interface::{Data, Read, Update, Write};
use io::M2RBSegmentWfeRms;

#[derive(Debug, Clone)]
pub struct M2RBLom(LinearOpticalModel);

impl From<LinearOpticalModel> for M2RBLom {
    fn from(value: LinearOpticalModel) -> Self {
        M2RBLom(value)
    }
}

impl Deref for M2RBLom {
    type Target = LinearOpticalModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for M2RBLom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Update for M2RBLom {
    fn update(&mut self) {
        self.0.update();
    }
}

impl Read<M2ASMReferenceBodyNodes> for M2RBLom {
    fn read(&mut self, data: Data<M2ASMReferenceBodyNodes>) {
        <LinearOpticalModel as Read<M2ASMReferenceBodyNodes>>::read(&mut self.0, data);
    }
}

impl Write<M2RBSegmentWfeRms> for M2RBLom {
    fn write(&mut self) -> Option<Data<M2RBSegmentWfeRms>> {
        <LinearOpticalModel as Write<M2RBSegmentWfeRms>>::write(&mut self.0)
    }
}
