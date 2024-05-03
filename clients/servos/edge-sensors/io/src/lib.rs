use gmt_dos_clients_io::optics::SegmentPiston;
use gmt_dos_clients_lom::LinearOpticalModel;
use interface::UID;

#[derive(UID)]
pub enum RBMCmd {}

#[derive(UID)]
pub enum M2EdgeSensorsAsRbms {}

#[derive(UID, Debug)]
#[uid(port = 55_001)]
pub enum M2ASMVoiceCoilsMotionAsRbms {}

#[derive(UID)]
#[uid(port = 55_002)]
pub enum M2S1Tz {}

#[derive(UID)]
#[uid(port = 55_003)]
pub enum M2S1VcAsTz {}

#[derive(UID)]
pub enum RbmAsShell {}

#[derive(UID, Debug)]
#[uid(port = 55_004)]
pub enum M2SegmentMeanActuator {}

#[derive(Debug)]
pub enum M1SegmentPiston {}
impl ::interface::UniqueIdentifier for M1SegmentPiston {
    const PORT: u16 = 55551;
    type DataType = <SegmentPiston<-9> as ::interface::UniqueIdentifier>::DataType;
}
impl ::interface::Write<M1SegmentPiston> for LinearOpticalModel {
    fn write(&mut self) -> Option<::interface::Data<M1SegmentPiston>> {
        <Self as ::interface::Write<SegmentPiston<-9>>>::write(self).map(|data| data.transmute())
    }
}

#[derive(Debug)]
pub enum M2SegmentPiston {}
impl ::interface::UniqueIdentifier for M2SegmentPiston {
    const PORT: u16 = 55552;
    type DataType = <SegmentPiston<-9> as ::interface::UniqueIdentifier>::DataType;
}
impl ::interface::Write<M2SegmentPiston> for LinearOpticalModel {
    fn write(&mut self) -> Option<::interface::Data<M2SegmentPiston>> {
        <Self as ::interface::Write<SegmentPiston<-9>>>::write(self).map(|data| data.transmute())
    }
}

#[derive(Debug)]
pub enum M2RBSegmentPiston {}
impl ::interface::UniqueIdentifier for M2RBSegmentPiston {
    const PORT: u16 = 55553;
    type DataType = <SegmentPiston<-9> as ::interface::UniqueIdentifier>::DataType;
}
impl ::interface::Write<M2RBSegmentPiston> for LinearOpticalModel {
    fn write(&mut self) -> Option<::interface::Data<M2RBSegmentPiston>> {
        <Self as ::interface::Write<SegmentPiston<-9>>>::write(self).map(|data| data.transmute())
    }
}

#[derive(Debug)]
pub enum M1SegmentWfeRms {}
impl ::interface::UniqueIdentifier for M1SegmentWfeRms {
    const PORT: u16 = 55561;
    type DataType = <SegmentPiston<-9> as ::interface::UniqueIdentifier>::DataType;
}
impl ::interface::Write<M1SegmentWfeRms> for LinearOpticalModel {
    fn write(&mut self) -> Option<::interface::Data<M1SegmentWfeRms>> {
        <Self as ::interface::Write<SegmentPiston<-9>>>::write(self).map(|data| data.transmute())
    }
}

#[derive(Debug)]
pub enum M2SegmentWfeRms {}
impl ::interface::UniqueIdentifier for M2SegmentWfeRms {
    const PORT: u16 = 55562;
    type DataType = <SegmentPiston<-9> as ::interface::UniqueIdentifier>::DataType;
}
impl ::interface::Write<M2SegmentWfeRms> for LinearOpticalModel {
    fn write(&mut self) -> Option<::interface::Data<M2SegmentWfeRms>> {
        <Self as ::interface::Write<SegmentPiston<-9>>>::write(self).map(|data| data.transmute())
    }
}

#[derive(Debug)]
pub enum M2RBSegmentWfeRms {}
impl ::interface::UniqueIdentifier for M2RBSegmentWfeRms {
    const PORT: u16 = 55563;
    type DataType = <SegmentPiston<-9> as ::interface::UniqueIdentifier>::DataType;
}
impl ::interface::Write<M2RBSegmentWfeRms> for LinearOpticalModel {
    fn write(&mut self) -> Option<::interface::Data<M2RBSegmentWfeRms>> {
        <Self as ::interface::Write<SegmentPiston<-9>>>::write(self).map(|data| data.transmute())
    }
}
