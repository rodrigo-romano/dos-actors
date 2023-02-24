//! GMT secondary mirror

use gmt_dos_clients::interface::UID;

/// M2 Rigid Body Motions
#[derive(UID)]
pub enum M2RigidBodyMotions {}
/// M2 Mode Shapes
#[derive(UID)]
pub enum M2ModeShape {}
/// M2 Positioner Forces
#[derive(UID)]
pub enum M2PositionerForces {}
/// M2 Positioner Nodes Displacements
#[derive(UID)]
pub enum M2PositionerNodes {}
pub mod fsm {
    use gmt_dos_clients::interface::UID;
    /// M2 FSM Piezo-Stack Actuators Forces
    #[derive(UID)]
    pub enum M2FSMPiezoForces {}
    /// M2 FSM Piezo-Stack Actuators Node Displacements
    #[derive(UID)]
    pub enum M2FSMPiezoNodes {}
    /// M2 FSM Tip-Tilt Modes
    #[derive(UID)]
    pub enum M2FSMTipTilt {}
}
pub mod asm {
    use gmt_dos_clients::interface::UID;
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
    pub mod segment {
        use gmt_dos_clients::interface::UniqueIdentifier;
        /// Voice coils forces
        pub enum VoiceCoilsForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for VoiceCoilsForces<ID> {
            type DataType = Vec<f64>;
        }
        /// Voice coil displacements
        pub enum VoiceCoilsMotion<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for VoiceCoilsMotion<ID> {
            type DataType = Vec<f64>;
        }
        /// Fluid damping forces
        pub enum FluidDampingForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for FluidDampingForces<ID> {
            type DataType = Vec<f64>;
        }
        /// Modal command coefficients
        pub enum ModalCommand<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for ModalCommand<ID> {
            type DataType = Vec<f64>;
        }
    }
}
