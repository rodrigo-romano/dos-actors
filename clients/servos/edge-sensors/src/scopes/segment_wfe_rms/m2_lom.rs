use std::ops::{Deref, DerefMut};

use gmt_dos_clients_io::gmt_m2::M2RigidBodyMotions;
use gmt_dos_clients_lom::LinearOpticalModel;
use interface::{Data, Read, Update, Write};
use io::M2SegmentWfeRms;

#[derive(Debug, Clone)]
pub struct M2Lom(LinearOpticalModel);

impl From<LinearOpticalModel> for M2Lom {
    fn from(value: LinearOpticalModel) -> Self {
        M2Lom(value)
    }
}

impl Deref for M2Lom {
    type Target = LinearOpticalModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for M2Lom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Update for M2Lom {
    fn update(&mut self) {
        self.0.update();
    }
}

impl Read<M2RigidBodyMotions> for M2Lom {
    fn read(&mut self, data: Data<M2RigidBodyMotions>) {
        <LinearOpticalModel as Read<M2RigidBodyMotions>>::read(&mut self.0, data);
    }
}

impl Write<M2SegmentWfeRms> for M2Lom {
    fn write(&mut self) -> Option<Data<M2SegmentWfeRms>> {
        <LinearOpticalModel as Write<M2SegmentWfeRms>>::write(&mut self.0)
    }
}
