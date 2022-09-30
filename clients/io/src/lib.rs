use dos_actors::{UniqueIdentifier, UID};

/// M1 Rigid Body Motions
#[derive(UID)]
pub enum M1RigidBodyMotions {}
/// M2 Rigid Body Motions
#[derive(UID)]
pub enum M2RigidBodyMotions {}
/// M1 Mode Shapes
#[derive(UID)]
pub enum M1ModeShapes {}
/// M2 Mode Shapes
#[derive(UID)]
pub enum M2ModeShape {}
/// Mount Encoders
#[derive(UID)]
pub enum MountEncoders {}
/// Mount Torques
#[derive(UID)]
pub enum MountTorques {}
/// Mount set point
#[derive(UID)]
pub enum MountSetPoint {}
/// M2 Positioner Forces
#[derive(UID)]
pub enum M2PositionerForces {}
/// M2 Positioner Nodes Displacements
#[derive(UID)]
pub enum M2PositionerNodes {}
/// M2 FSM Piezo-Stack Actuators Forces
#[derive(UID)]
pub enum M2FSMPiezoForces {}
/// M2 FSM Piezo-Stack Actuators Node Displacements
#[derive(UID)]
pub enum M2FSMPiezoNodes {}
/// M2 FSM Tip-Tilt Modes
#[derive(UID)]
pub enum M2FSMTipTilt {}
/// M2 ASM Rigid Body Forces
#[derive(UID)]
pub enum M2ASMRigidBodyForces {}
/// M2 ASM Cold Plate Forces
#[derive(UID)]
pub enum M2ASMColdPlateForces {}
/// M2 ASM Face Sheet Forces
#[derive(UID)]
pub enum M2ASMFaceSheetForces {}
/// M2 ASM Rigid Body Nodes
#[derive(UID)]
pub enum M2ASMRigidBodyNodes {}
/// M2 ASM Cold Plate Nodes
#[derive(UID)]
pub enum M2ASMColdPlateNodes {}
/// M2 ASM Face Sheet Nodes
#[derive(UID)]
pub enum M2ASMFaceSheetNodes {}
/// CFD Mount Wind Loads
#[derive(UID)]
pub enum CFDMountWindLoads {}
/// CFD M1 Loads
#[derive(UID)]
pub enum CFDM1WindLoads {}
/// CFD M2 Wind Loads
#[derive(UID)]
pub enum CFDM2WindLoads {}
/// M1 Hardpoints Forces
#[derive(UID)]
pub enum M1HardpointForces {}
/// M1 Hardpoints Nodes
#[derive(UID)]
pub enum M1HardpointNodes {}
/// M1 Segment Actuator Forces
#[derive(UID)]
pub enum M1SActuatorForces {}
