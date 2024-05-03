use std::ops::{Deref, DerefMut};

use gmt_dos_clients_io::gmt_m1::M1RigidBodyMotions;
use gmt_dos_clients_lom::LinearOpticalModel;
use interface::{Data, Read, Update, Write};
use io::M1SegmentPiston;

#[derive(Debug, Clone)]
pub struct M1Lom(LinearOpticalModel);

impl From<LinearOpticalModel> for M1Lom {
    fn from(value: LinearOpticalModel) -> Self {
        M1Lom(value)
    }
}

impl Deref for M1Lom {
    type Target = LinearOpticalModel;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for M1Lom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Update for M1Lom {
    fn update(&mut self) {
        self.0.update();
    }
}

impl Read<M1RigidBodyMotions> for M1Lom {
    fn read(&mut self, data: Data<M1RigidBodyMotions>) {
        <LinearOpticalModel as Read<M1RigidBodyMotions>>::read(&mut self.0, data);
    }
}

impl Write<M1SegmentPiston> for M1Lom {
    fn write(&mut self) -> Option<Data<M1SegmentPiston>> {
        <LinearOpticalModel as Write<M1SegmentPiston>>::write(&mut self.0)
    }
}
