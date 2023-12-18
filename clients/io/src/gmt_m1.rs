//! GMT primary mirror

use interface::UID;

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

pub mod assembly {
    use std::sync::Arc;

    use gmt_dos_actors::subsystem::gateway;
    use interface::UniqueIdentifier;

    use crate::Assembly;

    /// Rigid body motion (Tx,Ty,Tz,Rx,Ry,Rz)
    pub enum M1RigidBodyMotions {}
    impl Assembly for M1RigidBodyMotions {}
    impl UniqueIdentifier for M1RigidBodyMotions {
        type DataType = Vec<Arc<Vec<f64>>>;
        const PORT: u32 = 50_006;
    }

    /// Hardpoints displacements `[cell,mirror]`
    pub enum M1HardpointsMotion {}
    impl Assembly for M1HardpointsMotion {}
    impl UniqueIdentifier for M1HardpointsMotion {
        type DataType = Vec<Arc<Vec<f64>>>;
        const PORT: u32 = 50_007;
    }

    /// Hardpoints forces
    pub enum M1HardpointsForces {}
    impl Assembly for M1HardpointsForces {}
    impl UniqueIdentifier for M1HardpointsForces {
        type DataType = Vec<Arc<Vec<f64>>>;
        const PORT: u32 = 50_007;
    }

    /// Actuators command forces
    pub enum M1ActuatorCommandForces {}
    impl Assembly for M1ActuatorCommandForces {}
    impl UniqueIdentifier for M1ActuatorCommandForces {
        type DataType = Vec<Arc<Vec<f64>>>;
        const PORT: u32 = 50_008;
    }

    /// Actuators applied forces
    pub enum M1ActuatorAppliedForces {}
    impl Assembly for M1ActuatorAppliedForces {}
    impl UniqueIdentifier for M1ActuatorAppliedForces {
        type DataType = Vec<Arc<Vec<f64>>>;
        const PORT: u32 = 50_008;
    }

    // Mapping gateways data indices to inputs & output
    //  * In[0] -> M1RigidBodyMotions
    impl gateway::In for M1RigidBodyMotions {
        const IDX: usize = 0;
    }
    //  * In[1] -> M1ActuatorCommandForces
    impl gateway::In for M1ActuatorCommandForces {
        const IDX: usize = 1;
    }
    //  * In[2] -> M1HarpointsMotion
    impl gateway::In for M1HardpointsMotion {
        const IDX: usize = 2;
    }
    //  * Out[0] -> M1HardpointsForces
    impl gateway::Out for M1HardpointsForces {
        const IDX: usize = 0;
    }
    //  * Out[1] -> M1ActuatorAppliedForces
    impl gateway::Out for M1ActuatorAppliedForces {
        const IDX: usize = 1;
    }
}

/// Segment IO
pub mod segment {
    use gmt_dos_actors::subsystem::gateway;
    use interface::UniqueIdentifier;
    /// Force andf moment at center of gravity
    pub enum BarycentricForce<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for BarycentricForce<ID> {
        const PORT: u32 = 56_001 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Rigid body motion (Tx,Ty,Tz,Rx,Ry,Rz)
    pub enum RBM<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for RBM<ID> {
        const PORT: u32 = 56_002 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Hardpoints displacements `[cell,mirror]`
    pub enum HardpointsMotion<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for HardpointsMotion<ID> {
        const PORT: u32 = 56_003 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Hardpoints forces
    pub enum HardpointsForces<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for HardpointsForces<ID> {
        const PORT: u32 = 56_004 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Actuators applied forces
    pub enum ActuatorAppliedForces<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for ActuatorAppliedForces<ID> {
        const PORT: u32 = 56_005 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// Actuators command forces
    pub enum ActuatorCommandForces<const ID: u8> {}
    impl<const ID: u8> UniqueIdentifier for ActuatorCommandForces<ID> {
        const PORT: u32 = 56_006 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }
    /// SEGMENT RBM DOF selector ([0,...,6]->[Tx,Ty,Tz,Rx,Ry,Rz])
    pub enum M1S<const ID: u8, const DOF: u8> {}
    impl<const ID: u8, const DOF: u8> UniqueIdentifier for M1S<ID, DOF> {
        const PORT: u32 = 56_001 + 10 * (1 + DOF) as u32 + 100 * ID as u32;
        type DataType = Vec<f64>;
    }

    // Mapping gateways data indices to inputs & output
    //  * In[0] -> RBM<S>
    impl<const S: u8> gateway::In for RBM<S> {
        const IDX: usize = 0;
    }
    //  * In[1] -> ActuatorCommandForces<S>
    impl<const S: u8> gateway::In for ActuatorCommandForces<S> {
        const IDX: usize = 1;
    }
    //  * In[2] -> HardpointsMotion<S>
    impl<const S: u8> gateway::In for HardpointsMotion<S> {
        const IDX: usize = 2;
    }
    //  * Out[0] -> HardpointsForces<S>
    impl<const S: u8> gateway::Out for HardpointsForces<S> {
        const IDX: usize = 0;
    }
    //  * Out[1] -> ActuatorAppliedForces<S>
    impl<const S: u8> gateway::Out for ActuatorAppliedForces<S> {
        const IDX: usize = 1;
    }
}
