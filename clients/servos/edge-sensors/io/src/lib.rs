use gmt_dos_clients_io::optics::SegmentPiston;
use gmt_dos_clients_lom::LinearOpticalModel;
use interface::UID;

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
