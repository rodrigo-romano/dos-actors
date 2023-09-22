//! # Actors network
//!
//! The network module defines the interface to link actors to each other.

/// Interface for actors inputs
mod inputs;
pub use inputs::{AddActorInput, TryIntoInputs};

/// Interface for actors outputs
mod outputs;
pub use outputs::{ActorOutput, ActorOutputBuilder, AddActorOutput, AddOuput};

/// Definition of the payload between outputs and inputs
mod output_rx;
pub use output_rx::OutputRx;

/// Interface for actors log outputs
mod logs;
pub use logs::{IntoLogs, IntoLogsN};
