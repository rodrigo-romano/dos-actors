/*!
# Actor inputs and outputs implementation module

[Actor]s communicate using channels, one input of an [actor] send data through
either a [bounded] or an [unbounded] channel to an output of another actor.
The data that moves through a channel is encapsulated into a [Data] structure.

Each input and output has a reference to the [Actor] client that reads data from
 the input and write data to the output only if the client implements the [Read]
and [Write] traits.

[Actor]: crate::Actor
[bounded]: https://docs.rs/flume/latest/flume/fn.bounded
[unbounded]: https://docs.rs/flume/latest/flume/fn.unbounded
*/

use std::sync::Arc;

mod input;
pub(crate) use input::{Input, InputObject};
mod output;
pub(crate) use output::{Output, OutputObject};
pub type S<U> = Arc<crate::interface::Data<U>>;
