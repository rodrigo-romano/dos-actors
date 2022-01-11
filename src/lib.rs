//! # GMT Dynamic Optics Simulation Actors
//!
//! The GMT DOS `Actor`s are the building blocks of the GMT DOS integrated model.
//! Each `actor` has 3 components:
//!  1. **inputs**: `Option<Vec<InT>,Rx>`
//!  2. **outputs**: `Option<Vec<OutT>,Tx>`
//!  3. **state**: `(Option<Inputs>,Option<Outputs>>)`
//!
//! Inputs is a collection of [Input] and
//! outputs is a collection of [Output].
//! An actor must have at least either 1 [Input] or 1 [Output].
//! A pair of [Input]/[Output] is linked with a [channel](flume::unbounded) where the [Input] is the sender
//! and the [Output] is the receiver.
//! The same [Output] may be linked to several [Input]s.
//!
//! There are 3 kinds of [Actor]s:
//!  - **[Initiator]**: with only outputs
//!  - **[Terminator]**: with only inputs
//!  - **[Transformer]**: with both inputs and outputs
//!
//! Each [Actor] performs the same [task](Actor::task), within an infinite loop, consisting of 3 operations:
//!  1. [collect](Actor::collect) the inputs if any
//!  2. [compute](Actor::compute) the outputs if any based on the inputs
//!  3. [distribute](Actor::distribute) the outputs if any
//!
//! The loop exits when one of the following error happens: [ActorError::NoData], [ActorError::DropSend], [ActorError::DropRecv].
//!
//! All the [Input]s of an [Actor] are collected are the same rate, however [Output]s can be distributed at different rates.
//!

use flume::{Receiver, Sender};
use std::marker::PhantomData;

#[derive(thiserror::Error, Debug)]
pub enum ActorError {
    #[error("Receiver dropped")]
    DropRecv(#[from] flume::RecvError),
    #[error("Sender dropped")]
    DropSend(#[from] flume::SendError<()>),
    #[error("No new data produced")]
    NoData,
}
pub type Result<R> = std::result::Result<R, ActorError>;

/// [Actor](crate::Actor)s [Input]/[Output]
pub mod io {
    use crate::Result;
    use flume::{Receiver, Sender};
    use std::sync::Arc;

    type Data<T> = Arc<T>;
    /// [Actor](crate::Actor)s input
    pub struct Input<T: Default> {
        pub data: Data<T>,
        pub rx: Receiver<Data<T>>,
    }
    impl<T: Default> Input<T> {
        pub fn recv(&mut self) -> Result<&mut Self> {
            self.data = self.rx.recv()?;
            Ok(self)
        }
    }
    /// [Actor](crate::Actor)s output
    pub struct Output<T: Default> {
        pub data: Data<T>,
        pub tx: Vec<Sender<Data<T>>>,
    }
    impl<T: Default + Clone> Output<T> {
        pub fn send(&self) -> Result<&Self> {
            for tx in &self.tx {
                tx.send(self.data.clone())
                    .map_err(|_| flume::SendError(()))?;
            }
            Ok(self)
        }
    }
}

/// The kind of [Actor]s
pub mod actors_kind {
    /// [Actor](crate::Actor)s with only [Output](crate::Output)s
    pub struct Initiator;
    /// [Actor](crate::Actor)s with only [Input](crate::Input)s
    pub struct Terminator;
    /// [Actor](crate::Actor)s with both [Input](crate::Input)s and [Output](crate::Output)s
    pub struct Transformer;
}

use actors_kind::*;
use io::*;

type IO<S> = Vec<S>;
#[derive(Default)]
pub struct Actor<T, I, O, Kind>
where
    T: Default,
    I: Default,
    O: Default,
{
    pub channel: Option<(Sender<T>, Receiver<T>)>,
    pub inputs: Option<IO<Input<I>>>,
    pub outputs: Option<IO<Output<O>>>,
    kind: PhantomData<Kind>,
}

impl<T, I, O, Kind> Actor<T, I, O, Kind>
where
    T: Default,
    I: Default,
    O: Default + Clone,
{
    pub fn collect(&mut self) -> Result<&mut Self> {
        for input in self.inputs.as_mut().unwrap() {
            input.recv()?;
        }
        Ok(self)
    }
    pub fn distribute(&self) -> Result<&Self> {
        for output in self.outputs.as_ref().unwrap() {
            output.send()?;
        }
        Ok(self)
    }
    pub fn compute(&mut self) -> Result<&mut Self> {
        Ok(self)
    }
}

impl<T, I, O> Actor<T, I, O, Terminator>
where
    T: Default,
    I: Default,
    O: Default + Clone,
{
    pub fn new(inputs: IO<Input<I>>) -> Self {
        Self {
            channel: None,
            inputs: Some(inputs),
            outputs: None,
            kind: PhantomData,
        }
    }
    pub fn task(&mut self) -> Result<()> {
        loop {
            self.collect()?.compute()?;
        }
    }
}

impl<T, I, O> Actor<T, I, O, Initiator>
where
    T: Default,
    I: Default,
    O: Default + Clone,
{
    pub fn new(outputs: IO<Output<O>>) -> Self {
        Self {
            channel: None,
            inputs: None,
            outputs: Some(outputs),
            kind: PhantomData,
        }
    }
    pub fn task(&mut self) -> Result<()> {
        loop {
            self.compute()?.distribute()?;
        }
    }
}

impl<T, I, O> Actor<T, I, O, Transformer>
where
    T: Default,
    I: Default,
    O: Default + Clone,
{
    pub fn new(inputs: IO<Input<I>>, outputs: IO<Output<O>>) -> Self {
        Self {
            channel: None,
            inputs: Some(inputs),
            outputs: Some(outputs),
            kind: PhantomData,
        }
    }
    pub fn task(&mut self) -> Result<()> {
        loop {
            self.collect()?.compute()?.distribute()?;
        }
    }
}
/*

impl<I, O> Actor<I, O, Valid, Empty>
where
    I: Default,
    O: Default,
{
    pub fn new(outputs: IO<Output<O>>) -> Self {
        Self {
            channel: flume::bounded(CAP),
            inputs: None,
            outputs: Some(outputs),
            state_i: PhantomData,
            state_o: PhantomData,
        }
    }
}

impl<I, O> Actor<I, O, Empty, Empty>
where
    I: Default,
    O: Default,
{
    pub fn new(inputs: IO<Input<I>>, outputs: IO<Output<O>>) -> Self {
        Self {
            channel: flume::bounded(CAP),
            inputs: Some(inputs),
            outputs: Some(outputs),
            state_i: PhantomData,
            state_o: PhantomData,
        }
    }
}

impl<I, O, StateI> Actor<I, O, StateI, Valid>
where
    I: Default,
    O: Default,
{
    pub fn listen(&self) {
        let (_, rx) = &self.channel;
        while let Ok(data) = rx.recv() {
            // send output
        }
    }
}
impl<I, O> Actor<I, O, Valid, Empty>
where
    I: Default,
    O: Default,
{
    pub fn listen(&self) {
        let (_, rx) = &self.channel;
        while let Ok(data) = rx.recv() {
            // update state
            // send output
        }
    }
}
impl<I, O> Actor<I, O, Empty, Empty>
where
    I: Default,
    O: Default,
{
    pub fn listen(&self) {
        let (_, rx) = &self.channel;
        while let Ok(data) = rx.recv() {
            // get inputs
            // update state
            // send output
        }
    }
}
*/
