//! # GMT Dynamic Optics Simulation Actors
//!
//! The GMT DOS `Actor`s are the building blocks of the GMT DOS integrated model.
//! Each `actor` has 2 properties:
//!  1. **[inputs](Actor::inputs)**
//!  2. **[outputs](Actor::inputs)**
//!
//! [inputs](Actor::inputs) is a collection of [Input] and
//! [outputs](Actor::inputs) is a collection of [Output].
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
//! All the [Input]s of an [Actor] are collected are the same rate `NI`, and all the [Output]s are distributed at the same rate `NO`, however both [inputs](Actor::inputs) and [outputs](Actor::inputs) rates may be different.
//! For both [Transformer] and [Terminator] [Actor]s, [inputs](Actor::inputs) rate `NI` is inherited from the rate `NO` of [outputs](Actor::inputs) that the data is collected from i.e. `NI=NO`.
//! The rate `NI` or `NO` is defined as the ratio between the simulation sampling rate `[Hz]` and the actor sampling rate `[Hz]`, it must an integer ≥ 1.

use flume::{Receiver, Sender};
use std::{marker::PhantomData, ops::Deref, sync::Arc};

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
    use std::{ops::Deref, sync::Arc};

    /// [Input]/[Output] data
    ///
    /// `N` is the data transfer rate
    #[derive(Debug)]
    pub struct Data<T, const N: usize>(pub T);
    impl<T, const N: usize> Deref for Data<T, N> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<T: Clone, const N: usize> From<&Data<Vec<T>, N>> for Vec<T> {
        fn from(data: &Data<Vec<T>, N>) -> Self {
            data.to_vec()
        }
    }
    impl<T, const N: usize> From<Vec<T>> for Data<Vec<T>, N> {
        fn from(u: Vec<T>) -> Self {
            Data(u)
        }
    }

    pub type S<T, const N: usize> = Arc<Data<T, N>>;

    /// [Actor](crate::Actor)s input
    #[derive(Debug)]
    pub struct Input<T: Default, const N: usize> {
        pub data: S<T, N>,
        pub rx: Receiver<S<T, N>>,
    }
    impl<T: Default, const N: usize> Input<T, N> {
        pub fn new(data: T, rx: Receiver<S<T, N>>) -> Self {
            Self {
                data: Arc::new(Data(data)),
                rx,
            }
        }
        pub fn recv(&mut self) -> Result<&mut Self> {
            self.data = self.rx.recv()?;
            Ok(self)
        }
    }
    impl<T: Clone, const N: usize> From<&Input<Vec<T>, N>> for Vec<T> {
        fn from(input: &Input<Vec<T>, N>) -> Self {
            input.data.as_ref().into()
        }
    }
    /// [Actor](crate::Actor)s output
    #[derive(Debug)]
    pub struct Output<T: Default, const N: usize> {
        pub data: S<T, N>,
        pub tx: Vec<Sender<S<T, N>>>,
    }
    impl<T: Default, const N: usize> Output<T, N> {
        pub fn new(data: T, tx: Vec<Sender<S<T, N>>>) -> Self {
            Self {
                data: Arc::new(Data(data)),
                tx,
            }
        }
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
    #[derive(Debug)]
    pub struct Initiator;
    /// [Actor](crate::Actor)s with only [Input](crate::Input)s
    #[derive(Debug)]
    pub struct Terminator;
    /// [Actor](crate::Actor)s with both [Input](crate::Input)s and [Output](crate::Output)s
    #[derive(Debug)]
    pub struct Transformer;
}

use actors_kind::*;
use io::*;

type IO<S> = Vec<S>;
#[derive(Default, Debug)]
pub struct Actor<T, I, O, Kind, const NI: usize, const NO: usize>
where
    T: Default,
    I: Default,
    O: Default + std::fmt::Debug,
{
    pub channel: Option<(Sender<T>, Receiver<T>)>,
    pub inputs: Option<IO<Input<I, NI>>>,
    pub outputs: Option<IO<Output<O, NO>>>,
    time_idx: Arc<usize>,
    kind: PhantomData<Kind>,
}

impl<T, I, O, Kind, const NI: usize, const NO: usize> Actor<T, I, O, Kind, NI, NO>
where
    T: Default,
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    pub fn collect(&mut self) -> Result<&mut Self> {
        for input in self.inputs.as_mut().unwrap() {
            input.recv()?;
        }
        Ok(self)
    }
    pub fn distribute(&self) -> Result<&Self> {
        if self.time_idx.deref() % NO == 0 {
            for output in self.outputs.as_ref().unwrap() {
                output.send()?;
            }
        }
        Ok(self)
    }
    pub fn compute(&mut self) -> Result<&mut Self> {
        Ok(self)
    }
}

impl<T, I, O, const NI: usize, const NO: usize> Actor<T, I, O, Terminator, NI, NO>
where
    T: Default,
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    pub fn new(time_idx: Arc<usize>, inputs: IO<Input<I, NI>>) -> Self {
        Self {
            channel: None,
            inputs: Some(inputs),
            outputs: None,
            time_idx,
            kind: PhantomData,
        }
    }
    pub fn task(&mut self) -> Result<()> {
        loop {
            self.collect()?.compute()?;
        }
    }
}

impl<T, I, O, const NI: usize, const NO: usize> Actor<T, I, O, Initiator, NI, NO>
where
    T: Default,
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    pub fn new(time_idx: Arc<usize>, outputs: IO<Output<O, NO>>) -> Self {
        Self {
            channel: None,
            inputs: None,
            outputs: Some(outputs),
            time_idx,
            kind: PhantomData,
        }
    }
    pub fn task(&mut self) -> Result<()> {
        loop {
            self.compute()?.distribute()?;
        }
    }
}

impl<T, I, O, const NI: usize, const NO: usize> Actor<T, I, O, Transformer, NI, NO>
where
    T: Default,
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    pub fn new(time_idx: Arc<usize>, inputs: IO<Input<I, NI>>, outputs: IO<Output<O, NO>>) -> Self {
        Self {
            channel: None,
            inputs: Some(inputs),
            outputs: Some(outputs),
            time_idx,
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
