//! GMT primary mirror

use gmt_dos_clients::interface::UID;

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
    use gmt_dos_clients::interface::UniqueIdentifier;
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
