//! GMT secondary mirror

use interface::UID;

/// M2 Rigid Body Motions
#[derive(UID)]
#[uid(port = 57_001)]
pub enum M2RigidBodyMotions {}
/// M2 Mode Shapes
#[derive(UID)]
pub enum M2ModeShape {}
/// M2 Positioner Forces
#[derive(UID)]
#[uid(port = 57_002)]
pub enum M2PositionerForces {}
/// M2 Positioner Nodes Displacements
#[derive(UID)]
#[uid(port = 57_003)]
pub enum M2PositionerNodes {}
pub mod fsm {
    use interface::UID;
    /// M2 FSM Piezo-Stack Actuators Forces
    #[derive(UID)]
    #[uid(port = 58_001)]
    pub enum M2FSMPiezoForces {}
    /// M2 FSM Piezo-Stack Actuators Node Displacements
    #[derive(UID)]
    #[uid(port = 58_002)]
    pub enum M2FSMPiezoNodes {}
    /// M2 FSM Tip-Tilt Modes
    #[derive(UID)]
    #[uid(port = 58_003)]
    pub enum M2FSMTipTilt {}
}
pub mod asm {
    use interface::UID;
    /// M2 ASM Rigid Body Forces
    #[derive(UID)]
    #[uid(port = 59_001)]
    pub enum M2ASMRigidBodyForces {}
    /// M2 ASM Cold Plate Forces
    #[derive(UID)]
    #[uid(port = 59_002)]
    pub enum M2ASMColdPlateForces {}
    /// M2 ASM Face Sheet Forces
    #[derive(UID)]
    #[uid(port = 59_003)]
    pub enum M2ASMFaceSheetForces {}
    /// M2 ASM Rigid Body Nodes
    #[derive(UID)]
    #[uid(port = 59_004)]
    pub enum M2ASMRigidBodyNodes {}
    /// M2 ASM Cold Plate Nodes
    #[derive(UID)]
    #[uid(port = 59_005)]
    pub enum M2ASMColdPlateNodes {}
    /// M2 ASM Face Sheet Nodes
    #[derive(UID)]
    #[uid(port = 59_006)]
    pub enum M2ASMFaceSheetNodes {}
    pub mod segment {
        use interface::UniqueIdentifier;
        /// Voice coils forces
        pub enum VoiceCoilsForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for VoiceCoilsForces<ID> {
            const PORT: u32 = 59_0001 + 100 * ID as u32;
            type DataType = Vec<f64>;
        }
        /// Voice coil displacements
        pub enum VoiceCoilsMotion<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for VoiceCoilsMotion<ID> {
            const PORT: u32 = 59_0002 + 100 * ID as u32;
            type DataType = Vec<f64>;
        }
        /// Fluid damping forces
        pub enum FluidDampingForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for FluidDampingForces<ID> {
            const PORT: u32 = 59_0003 + 100 * ID as u32;
            type DataType = Vec<f64>;
        }
        /// Modal command coefficients
        pub enum AsmCommand<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for AsmCommand<ID> {
            const PORT: u32 = 59_0004 + 100 * ID as u32;
            type DataType = Vec<f64>;
        }
        /// Face sheet displacements
        pub enum FaceSheetFigure<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for FaceSheetFigure<ID> {
            const PORT: u32 = 59_0005 + 100 * ID as u32;
            type DataType = Vec<f64>;
        }
    }
}
