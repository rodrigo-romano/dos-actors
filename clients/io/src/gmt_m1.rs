//! GMT primary mirror

use gmt_dos_clients::interface::UID;

/// M1 Rigid Body Motions
#[derive(UID)]
#[uid(port = 56_001)]
pub enum M1RigidBodyMotions {}
/// M1 Mode Shapes
#[derive(UID)]
#[uid(port = 56_002)]
pub enum M1ModeShapes {}
/// M1 Hardpoints Forces
#[derive(UID)]
#[uid(port = 56_003)]
pub enum M1HardpointForces {}
/// M1 Hardpoints Nodes
#[derive(UID)]
#[uid(port = 56_004)]
pub enum M1HardpointNodes {}
/// M1 Segment Actuator Forces
#[derive(UID)]
#[uid(port = 56_005)]
pub enum M1SActuatorForces {}
/// Segment IO
pub mod segment {
    use gmt_dos_clients::interface::UniqueIdentifier;
    /// Force andf moment at center of gravity
    pub enum BarycentricForce<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for BarycentricForce<ID> {
        const PORT: u32 = 56_0001 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Rigid body motion (Tx,Ty,Tz,Rx,Ry,Rz)
    pub enum RBM<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for RBM<ID> {
        const PORT: u32 = 56_0002 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Hardpoints displacements `[cell,mirror]`
    pub enum HardpointsMotion<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for HardpointsMotion<ID> {
        const PORT: u32 = 56_0003 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Hardpoints forces
    pub enum HardpointsForces<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for HardpointsForces<ID> {
        const PORT: u32 = 56_0004 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Actuators applied forces
    pub enum ActuatorAppliedForces<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for ActuatorAppliedForces<ID> {
        const PORT: u32 = 56_0005 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Actuators command forces
    pub enum ActuatorCommandForces<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for ActuatorCommandForces<ID> {
        const PORT: u32 = 56_0006 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
}
