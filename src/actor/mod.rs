/*! # Actor model

The module provides an implementation of the [actor model](https://youtu.be/ELwEdb_pD0k) for the GMT Integrated Model.

Actors provide the functionalities for [client]s to exchange information and to update [client]s state, based on the data received from other [client]s through these [client]s own Actors.

An [Actor] is build first from a client, then outputs are added one by one and for each output a corresponding input is created that is added to another [Actor].
If the output and input rates of an [Actor] are not specified, they are set to 1.

A model will always have a least one [Actor] without inputs, the [Initiator], and one [Actor] without outputs, the [Terminator].

An [Actor] runs a loop inside a dedicated thread.
The loop starts waiting for new inputs, upon reception the client reads the inputs, update its state and write to the outputs.
The [Actor] will either send the outputs immediately into the buffer of the output/input channel or, if the buffer is full, it will wait until the buffer has been read by the receiving input.

An actor can simply be derived from a client with the [From](crate::Actor::from) trait.
Note that the client is consumed and no longer available.
```
use dos_actors::prelude::*;
let source: Initiator<_> = Signals::new(1, 100).into();
```

If the client must remain available for later use, it must be wrapped inside a [Mutex] within an [Arc].
This can be easily done with the [into_arcx] method of the [ArcMutex] trait that has a blanket implementation for all type that implements the [Update] trait.
```
use dos_actors::prelude::*;
let logging = Logging::<f64>::default().into_arcx();
let sink = Terminator::<_>::new(logging);
```

[client]: crate::clients
[Mutex]: tokio::sync::Mutex
[Arc]: std::sync::Arc
[Arcmutex]: crate::ArcMutex
[into_arcx]: crate::ArcMutex::into_arcx
[Signals]: crate::clients::Signals
[Sampler]: crate::clients::Sampler
[Logging]: crate::clients::Logging
*/

use crate::Result;
use async_trait::async_trait;
mod actor;
pub use actor::Actor;

/// Actor client state update interface
pub trait Update {
    fn update(&mut self) {}
}

/// Type alias for an actor without outputs
pub type Terminator<C, const NI: usize = 1> = Actor<C, NI, 0>;
/// Type alias for an actor without inputs
pub type Initiator<C, const NO: usize = 1> = Actor<C, 0, NO>;

#[async_trait]
pub trait Task: Send {
    /// Runs the [Actor] infinite loop
    ///
    /// The loop ends when the client data is [None] or when either the sending of receiving
    /// end of a channel is dropped
    async fn async_run(&mut self) -> Result<()>;
    /// Run the actor loop in a dedicated thread
    fn spawn(self) -> tokio::task::JoinHandle<()>;
    /**
    Validates the inputs

    Returns en error if there are some inputs but the inputs rate is zero
    or if there are no inputs and the inputs rate is positive
    */
    fn check_inputs(&self) -> Result<()>;
    /**
    Validates the outputs

    Returns en error if there are some outputs but the outputs rate is zero
    or if there are no outputs and the outputs rate is positive
    */
    fn check_outputs(&self) -> Result<()>;
    /// Run the actor loop
    async fn task(&mut self);
}
