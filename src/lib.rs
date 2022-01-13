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
//! A pair of [Input]/[Output] is linked with a [channel](flume::bounded) where the [Input] is the sender
//! and the [Output] is the receiver.
//! The same [Output] may be linked to several [Input]s.
//!
//! There are 2 uniques of [Actor]s:
//!  - **[Initiator]**: with only outputs
//!  - **[Terminator]**: with only inputs
//!
//! Each [Actor] performs the same [task](Actor::task), within an infinite loop, consisting of 3 operations:
//!  1. [collect](Actor::collect) the inputs if any
//!  2. [compute](Actor::compute) the outputs if any based on the inputs
//!  3. [distribute](Actor::distribute) the outputs if any
//!
//! The loop exits when one of the following error happens: [ActorError::NoData], [ActorError::DropSend], [ActorError::DropRecv].
//!
//! All the [Input]s of an [Actor] are collected are the same rate `NI`, and all the [Output]s are distributed at the same rate `NO`, however both [inputs](Actor::inputs) and [outputs](Actor::inputs) rates may be different.
//! The [inputs](Actor::inputs) rate `NI` is inherited from the rate `NO` of [outputs](Actor::outputs) that the data is collected from i.e. `NI=NO`.
//! The rate `NI` or `NO` is defined as the ratio between the simulation sampling rate `[Hz]` and the actor sampling rate `[Hz]`, it must be an integer ≥ 1.
//! If `NI>NO`, [outputs](Actor::outputs) are upsampled with a simple sample-and-hold for `NI/NO` samples.
//! If `NO>NI`, [outputs](Actor::outputs) are decimated by a factor `NO/NI`

use std::{marker::PhantomData, ops::Deref, sync::Arc};

#[derive(thiserror::Error, Debug)]
pub enum ActorError {
    #[error("Receiver dropped")]
    DropRecv(#[from] flume::RecvError),
    #[error("Sender dropped")]
    DropSend(#[from] flume::SendError<()>),
    #[error("No new data produced")]
    NoData,
    #[error("No inputs defined")]
    NoInputs,
    #[error("No outputs defined")]
    NoOutputs,
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

use io::*;

type IO<S> = Vec<S>;
#[derive(Default, Debug)]
pub struct Actor<I, O, const NI: usize, const NO: usize>
where
    I: Default,
    O: Default + std::fmt::Debug,
{
    pub inputs: Option<IO<Input<I, NI>>>,
    pub outputs: Option<IO<Output<O, NO>>>,
    time_idx: Arc<usize>,
}

impl<I, O, const NI: usize, const NO: usize> Actor<I, O, NI, NO>
where
    I: Default + std::fmt::Debug,
    O: Default + std::fmt::Debug,
{
    pub fn new(time_idx: Arc<usize>, inputs: IO<Input<I, NI>>, outputs: IO<Output<O, NO>>) -> Self {
        Self {
            inputs: Some(inputs),
            outputs: Some(outputs),
            time_idx,
        }
    }
    pub fn collect(&mut self) -> Result<&mut Self> {
        for input in self.inputs.as_mut().ok_or(ActorError::NoInputs)? {
            input.recv()?;
        }
        Ok(self)
    }
    pub fn distribute(&self) -> Result<&Self> {
        if self.time_idx.deref() % NO == 0 {
            for output in self.outputs.as_ref().ok_or(ActorError::NoOutputs)? {
                output.send()?;
            }
        }
        Ok(self)
    }
    pub fn compute(&mut self) -> Result<&mut Self> {
        Ok(self)
    }
    pub fn task(&mut self) -> Result<()> {
        match (self.inputs.as_ref(), self.outputs.as_ref()) {
            (Some(_), Some(_)) => {
                if NO >= NI {
                    loop {
                        for _ in 0..NO / NI {
                            self.collect()?.compute()?;
                        }
                        self.distribute()?;
                    }
                } else {
                    loop {
                        self.collect()?.compute()?;
                        for _ in 0..NI / NO {
                            self.distribute()?;
                        }
                    }
                }
            }
            (None, Some(_)) => loop {
                self.compute()?.distribute()?;
            },
            (Some(_), None) => loop {
                self.collect()?.compute()?;
            },
            (None, None) => Ok(()),
        }
    }
}

pub struct Terminator<I, const NI: usize>(PhantomData<I>);
impl<I, const NI: usize> Terminator<I, NI>
where
    I: Default + std::fmt::Debug,
{
    pub fn new(time_idx: Arc<usize>, inputs: IO<Input<I, NI>>) -> Actor<I, (), NI, 0> {
        Actor {
            inputs: Some(inputs),
            outputs: None,
            time_idx,
        }
    }
}

pub struct Initiator<O, const NO: usize>(PhantomData<O>);
impl<O, const NO: usize> Initiator<O, NO>
where
    O: Default + std::fmt::Debug,
{
    pub fn new(time_idx: Arc<usize>, outputs: IO<Output<O, NO>>) -> Actor<(), O, 0, NO> {
        Actor {
            inputs: None,
            outputs: Some(outputs),
            time_idx,
        }
    }
}
