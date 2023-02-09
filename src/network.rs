use std::sync::Arc;

use crate::{
    io::{self, Assoc, UniqueIdentifier, Update},
    Actor, Result,
};
use async_trait::async_trait;

mod inputs;
mod outputs;

/// Assign inputs to actors
pub trait IntoInputs<'a, T, U, CO, const NO: usize, const NI: usize>
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
    CO: 'static + Update + Send + io::Write<U>,
{
    /// Creates a new input for 'actor' from the last 'Receiver'
    #[must_use = r#"append ".ok()" to squash the "must use" warning"#]
    fn into_input<CI, const N: usize>(self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        CI: 'static + Update + Send + io::Read<U>,
        Self: Sized;
    /// Returns an error if there are any unassigned receivers
    ///
    /// Otherwise return the actor with the new output
    fn ok(self) -> Result<&'a mut Actor<CO, NI, NO>>
    where
        Self: Sized;
}

pub struct OutputActorRx<'a, C, U, const NI: usize, const NO: usize>(
    (&'a mut Actor<C, NI, NO>, Vec<Rx<U>>),
)
where
    C: Update + Send,
    U: UniqueIdentifier + Send + Sync;
pub trait TryIntoInputs<'a, T, U, CO, const NO: usize, const NI: usize>
where
    T: 'static + Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier<DataType = T>,
    CO: 'static + Update + Send + io::Write<U>,
{
    /// Try to create a new input for 'actor' from the last 'Receiver'
    fn try_into_input<CI, const N: usize>(self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        CI: 'static + Update + Send + io::Read<U>,
        Self: Sized;
}

/// Interface for data logging types
pub trait Entry<U: UniqueIdentifier> {
    /// Adds an entry to the logger
    fn entry(&mut self, size: usize);
}
/// Assign a new entry to a logging actor
#[async_trait]
pub trait IntoLogsN<CI, const N: usize, const NO: usize>
where
    CI: Update + Send,
{
    async fn logn(self, actor: &mut Actor<CI, NO, N>, size: usize) -> Self
    where
        Self: Sized;
}

/// Assign a new entry to a logging actor
#[async_trait]
pub trait IntoLogs<CI, const N: usize, const NO: usize>
where
    CI: Update + Send,
{
    async fn log(self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        Self: Sized;
}

/// Actor outputs builder
pub struct ActorOutputBuilder {
    capacity: Vec<usize>,
    bootstrap: bool,
}

type Rx<U> = flume::Receiver<Arc<io::Data<U>>>;

/// Actor add output interface
pub trait AddOuput<'a, C, const NI: usize, const NO: usize>
where
    C: 'static + Update + Send,
{
    /// Sets the channel to unbounded
    fn unbounded(self) -> Self;
    /// Flags the output to be bootstrapped
    fn bootstrap(self) -> Self;
    /// Multiplexes the output `n` times
    fn multiplex(self, n: usize) -> Self;
    /// Builds the new output
    fn build<U>(self) -> (&'a mut Actor<C, NI, NO>, Vec<Rx<U>>)
    where
        C: io::Write<U>,
        U: 'static + UniqueIdentifier + Send + Sync,
        Assoc<U>: Send + Sync;
    /// Try to build a new output where you must fail to succeed
    fn try_build<U>(self) -> std::result::Result<(), OutputActorRx<'a, C, U, NI, NO>>
    where
        C: io::Write<U>,
        U: 'static + UniqueIdentifier + Send + Sync,
        Assoc<U>: Send + Sync,
        Self: Sized,
    {
        std::result::Result::Err(OutputActorRx(<Self as AddOuput<C, NI, NO>>::build(self)))
    }
}
