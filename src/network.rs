use crate::interface::{self as io, Assoc, UniqueIdentifier, Update};
use crate::{Actor, Result};
use async_trait::async_trait;
use std::sync::Arc;

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
    fn legacy_into_input<CI, const N: usize>(self, actor: &mut Actor<CI, NO, N>) -> Self
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

pub struct OutputRx<U, C, const NI: usize, const NO: usize>
where
    U: UniqueIdentifier + Send + Sync,
    C: Update + io::Write<U>,
{
    actor: String,
    output: String,
    hash: u64,
    rxs: Vec<Rx<U>>,
    client: Arc<tokio::sync::Mutex<C>>,
}
pub trait TryIntoInputs<U, CO, const NO: usize, const NI: usize>
where
    Assoc<U>: Send + Sync,
    U: 'static + Send + Sync + UniqueIdentifier,
    CO: 'static + Update + Send + io::Write<U>,
{
    /// Try to create a new input for 'actor' from the last 'Receiver'
    fn into_input<CI, const N: usize>(self, actor: &mut Actor<CI, NO, N>) -> Self
    where
        CI: 'static + Update + Send + io::Read<U>,
        Self: Sized;
}

/// Assign a new entry to a logging actor
#[async_trait]
pub trait IntoLogsN<CI, const N: usize, const NO: usize>
where
    CI: Update + Send,
{
    async fn logn(mut self, actor: &mut Actor<CI, NO, N>, size: usize) -> Self
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
    fn legacy_build<U>(self) -> (&'a mut Actor<C, NI, NO>, Vec<Rx<U>>)
    where
        C: io::Write<U>,
        U: 'static + UniqueIdentifier + Send + Sync,
        Assoc<U>: Send + Sync;
    /// Try to build a new output where you must fail to succeed
    fn build<U>(self) -> std::result::Result<(), OutputRx<U, C, NI, NO>>
    where
        C: io::Write<U>,
        U: 'static + UniqueIdentifier + Send + Sync,
        Assoc<U>: Send + Sync,
        Self: Sized;
}
