use interface::UID;

#[derive(UID)]
pub enum EdgeSensorsAsRbms {}

#[derive(UID)]
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
