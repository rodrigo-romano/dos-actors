/// GMT primary mirror
pub mod gmt_m1 {
    use gmt_dos_actors_interface::UID;
    /// M1 Rigid Body Motions
    #[derive(UID)]
    pub enum M1RigidBodyMotions {}
    /// M1 Mode Shapes
    #[derive(UID)]
    pub enum M1ModeShapes {}
    /// M1 Hardpoints Forces
    #[derive(UID)]
    pub enum M1HardpointForces {}
    /// M1 Hardpoints Nodes
    #[derive(UID)]
    pub enum M1HardpointNodes {}
    /// M1 Segment Actuator Forces
    #[derive(UID)]
    pub enum M1SActuatorForces {}
    /// Segment IO
    pub mod segment {
        // use gmt_dos_actors_interface::UID;
        use gmt_dos_actors_interface::UniqueIdentifier;
        /// Force andf moment at center of gravity
        pub enum BarycentricForce<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for BarycentricForce<ID> {
            type DataType = Vec<f64>;
        }
        /// Rigid body motion (Tx,Ty,Tz,Rx,Ry,Rz)
        pub enum RBM<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for RBM<ID> {
            type DataType = Vec<f64>;
        }
        /// Hardpoints displacements [cell,mirror]
        pub enum HardpointsMotion<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for HardpointsMotion<ID> {
            type DataType = Vec<f64>;
        }
        /// Hardpoints forces
        pub enum HardpointsForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for HardpointsForces<ID> {
            type DataType = Vec<f64>;
        }
        /// Actuators applied forces
        pub enum ActuatorAppliedForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for ActuatorAppliedForces<ID> {
            type DataType = Vec<f64>;
        }
        /// Actuators command forces
        pub enum ActuatorCommandForces<const ID: u8> {}
        impl<const ID: u8> UniqueIdentifier for ActuatorCommandForces<ID> {
            type DataType = Vec<f64>;
        }
    }
}
/// GMT secondary mirror
pub mod gmt_m2 {
    use gmt_dos_actors_interface::UID;
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
        use gmt_dos_actors_interface::UID;
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
        use gmt_dos_actors_interface::UID;
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
    }
}
/// Mount
pub mod mount {
    use gmt_dos_actors_interface::UID;
    /// Mount Encoders
    #[derive(UID)]
    pub enum MountEncoders {}
    /// Mount Torques
    #[derive(UID)]
    pub enum MountTorques {}
    /// Mount set point
    #[derive(UID)]
    pub enum MountSetPoint {}
}
/// CFD wind loads
pub mod cfd_wind_loads {
    use gmt_dos_actors_interface::UID;
    /// CFD Mount Wind Loads
    #[derive(UID)]
    pub enum CFDMountWindLoads {}
    /// CFD M1 Loads
    #[derive(UID)]
    pub enum CFDM1WindLoads {}
    /// CFD M2 Wind Loads
    #[derive(UID)]
    pub enum CFDM2WindLoads {}
}
