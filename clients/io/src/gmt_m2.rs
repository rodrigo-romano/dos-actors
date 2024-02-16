//! GMT secondary mirror

use interface::UID;

/// M2 Rigid Body Motions
#[derive(UID)]
#[uid(port = 57_001)]
pub enum M2RigidBodyMotions {}
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
// Adaptive Secondary Mirror IO
pub mod asm {
    use interface::{UniqueIdentifier, UID};
    use std::sync::Arc;

    use crate::Assembly;

    /// M2 ASM Rigid Body Forces
    #[derive(UID)]
    #[uid(port = 59_001)]
    pub enum M2ASMReferenceBodyForces {}
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
    pub enum M2ASMReferenceBodyNodes {}
    /// M2 ASM Cold Plate Nodes
    #[derive(UID)]
    #[uid(port = 59_005)]
    pub enum M2ASMColdPlateNodes {}
    /// M2 ASM Face Sheet Nodes
    #[derive(UID)]
    #[uid(port = 59_006)]
    pub enum M2ASMFaceSheetNodes {}

    /// M2 ASM voice coils forces
    pub enum M2ASMVoiceCoilsForces {}
    impl Assembly for M2ASMVoiceCoilsForces {}
    impl UniqueIdentifier for M2ASMVoiceCoilsForces {
        type DataType = Vec<Arc<Vec<f64>>>;
        const PORT: u16 = 50_007;
    }

    /// M2 ASM voice coils displacements
    pub enum M2ASMVoiceCoilsMotion {}
    impl Assembly for M2ASMVoiceCoilsMotion {}
    impl UniqueIdentifier for M2ASMVoiceCoilsMotion {
        type DataType = Vec<Arc<Vec<f64>>>;
        const PORT: u16 = 50_008;
    }

    /// M2 ASM fluid damping forces
    pub enum M2ASMFluidDampingForces {}
    impl Assembly for M2ASMFluidDampingForces {}
    impl UniqueIdentifier for M2ASMFluidDampingForces {
        type DataType = Vec<Arc<Vec<f64>>>;
        const PORT: u16 = 50_009;
    }

    /// M2 ASM modal command coefficients
    pub enum M2ASMAsmCommand {}
    impl Assembly for M2ASMAsmCommand {}
    impl UniqueIdentifier for M2ASMAsmCommand {
        type DataType = Vec<f64>;
        const PORT: u16 = 50_010;
    }

    /// M2 ASM face sheet displacements
    pub enum M2ASMFaceSheetFigure {}
    impl Assembly for M2ASMFaceSheetFigure {}
    impl UniqueIdentifier for M2ASMFaceSheetFigure {
        type DataType = Vec<Vec<f64>>;
        const PORT: u16 = 50_011;
    }

    pub mod segment {
        use interface::UniqueIdentifier;
        /// Voice coils forces
        pub enum VoiceCoilsForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for VoiceCoilsForces<ID> {
            const PORT: u16 = 59_001 + 100 * ID as u16;
            type DataType = Vec<f64>;
        }
        /// Voice coils displacements
        pub enum VoiceCoilsMotion<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for VoiceCoilsMotion<ID> {
            const PORT: u16 = 59_002 + 100 * ID as u16;
            type DataType = Vec<f64>;
        }
        /// Fluid damping forces
        pub enum FluidDampingForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for FluidDampingForces<ID> {
            const PORT: u16 = 59_003 + 100 * ID as u16;
            type DataType = Vec<f64>;
        }
        /// Modal command coefficients
        pub enum AsmCommand<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for AsmCommand<ID> {
            const PORT: u16 = 59_004 + 100 * ID as u16;
            type DataType = Vec<f64>;
        }
        /// Face sheet displacements
        pub enum FaceSheetFigure<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for FaceSheetFigure<ID> {
            const PORT: u16 = 59_005 + 100 * ID as u16;
            type DataType = Vec<f64>;
        }
    }
}
