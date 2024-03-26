use gmt_dos_clients_fem::fem_io::actors_outputs::{M2EdgeSensors, OSSM1EdgeSensors};
use gmt_dos_clients_fem::{DiscreteStateSpace, ExponentialMatrix};

use super::Include;

#[derive(Debug, Clone)]
struct M1EdgeSensor;
#[derive(Debug, Clone)]
struct M2EdgeSensor;

/**
Edge sensors builder

The edge sensors builder adds the following outputs to the FEM:
 * [`M1EdgeSensors`](gmt_dos_clients_io::gmt_m1::M1EdgeSensors)
 * [`M2EdgeSensors`](gmt_dos_clients_io::gmt_m2::M2EdgeSensors)

 ```no_run
use gmt_dos_clients_servos::{GmtServoMechanisms, EdgeSensors};
use gmt_fem::FEM;

const ACTUATOR_RATE: usize = 80; // 100Hz

let frequency = 8000_f64; // Hz
let fem = FEM::from_env()?;

let gmt_servos =
    GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
        .edge_sensors(EdgeSensors::both())
        .build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
 */
#[derive(Debug, Default, Clone)]
pub struct EdgeSensors {
    m1: Option<M1EdgeSensor>,
    m2: Option<M2EdgeSensor>,
}

impl EdgeSensors {
    /// Creates a new empty [EdgeSensors] builder
    /// ```no_run
    /// # use gmt_dos_clients_servos::{GmtServoMechanisms, EdgeSensors};
    /// # use gmt_fem::FEM;
    /// # const ACTUATOR_RATE: usize = 80; // 100Hz
    /// # let frequency = 8000_f64; // Hz
    /// # let fem = FEM::from_env()?;
    /// let gmt_servos =
    ///     GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
    ///         .edge_sensors(EdgeSensors::none())
    ///         .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn none() -> Self {
        Self { m1: None, m2: None }
    }
    /// Creates a new [EdgeSensors] builder for both M1 and M2 edge sensors
    /// ```no_run
    /// # use gmt_dos_clients_servos::{GmtServoMechanisms, EdgeSensors};
    /// # use gmt_fem::FEM;
    /// # const ACTUATOR_RATE: usize = 80; // 100Hz
    /// # let frequency = 8000_f64; // Hz
    /// # let fem = FEM::from_env()?;
    /// let gmt_servos =
    ///     GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
    ///         .edge_sensors(EdgeSensors::both())
    ///         .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn both() -> Self {
        Self {
            m1: Some(M1EdgeSensor),
            m2: Some(M2EdgeSensor),
        }
    }
    /// Creates a new [EdgeSensors] builder for M1 edge sensors
    /// ```no_run
    /// # use gmt_dos_clients_servos::{GmtServoMechanisms, EdgeSensors};
    /// # use gmt_fem::FEM;
    /// # const ACTUATOR_RATE: usize = 80; // 100Hz
    /// # let frequency = 8000_f64; // Hz
    /// # let fem = FEM::from_env()?;
    /// let gmt_servos =
    ///     GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
    ///         .edge_sensors(EdgeSensors::m1())
    ///         .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn m1() -> Self {
        Self {
            m1: Some(M1EdgeSensor),
            m2: None,
        }
    }
    /// Creates a new [EdgeSensors] builder for M2 edge sensors
    /// ```no_run
    /// # use gmt_dos_clients_servos::{GmtServoMechanisms, EdgeSensors};
    /// # use gmt_fem::FEM;
    /// # const ACTUATOR_RATE: usize = 80; // 100Hz
    /// # let frequency = 8000_f64; // Hz
    /// # let fem = FEM::from_env()?;
    /// let gmt_servos =
    ///     GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
    ///         .edge_sensors(EdgeSensors::m2())
    ///         .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn m2() -> Self {
        Self {
            m1: None,
            m2: Some(M2EdgeSensor),
        }
    }
}

impl<'a> Include<'a, EdgeSensors> for DiscreteStateSpace<'a, ExponentialMatrix> {
    fn including(
        self,
        edge_sensors: Option<&'a mut EdgeSensors>,
    ) -> Result<Self, gmt_dos_clients_fem::StateSpaceError> {
        let Some(edge_sensors) = edge_sensors else {
            return Ok(self);
        };
        let mut state_space = self;
        if edge_sensors.m1.is_some() {
            state_space = state_space.outs::<OSSM1EdgeSensors>();
        }
        if edge_sensors.m2.is_some() {
            state_space = state_space.outs::<M2EdgeSensors>();
        }
        Ok(state_space)
    }
}
