/*!
# Network framework

The network module defines the interface to link actors to each other.

![Output to input](https://raw.githubusercontent.com/rconan/dos-actors/main/actors/src/framework/network/out2in.dot.png)

![Trait bounds](https://raw.githubusercontent.com/rconan/dos-actors/main/actors/src/framework/network/bounds.dot.png)

*/

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
