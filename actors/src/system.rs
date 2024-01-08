use std::marker::PhantomData;
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::{
    actor::{Actor, PlainActor},
    framework::{
        model::{CheckError, SystemFlowChart, TaskError},
        network::{ActorOutput, AddActorInput},
    },
    prelude::{AddActorOutput, GetName, Model, Unknown},
    Check, Task,
};

// mod check;
// mod flowchart;
// mod task;

/// An actors sub-[Model](crate::model::Model)
pub trait System: Sized + Clone + Display + Send + Sync + GetName {
    fn name(&self) -> String {
        String::from("SYSTEM")
    }
    fn build(&mut self) -> anyhow::Result<&mut Self>;
    fn plain(&self) -> PlainActor;
}

impl<T: System> GetName for T {
    fn get_name(&self) -> String {
        self.name()
    }
}

#[async_trait::async_trait]
impl<T> Task for Sys<T>
where
    T: System,
    for<'a> &'a T: IntoIterator<Item = Box<&'a dyn Check>>,
    Box<T>: IntoIterator<Item = Box<dyn Task>>,
{
    async fn async_run(&mut self) -> std::result::Result<(), TaskError> {
        todo!()
    }

    async fn task(mut self: Box<Self>) -> std::result::Result<(), TaskError> {
        let name = self.name();
        let q = *self;
        let w = q.sys;
        let b = Box::new(w);
        Model::<Unknown>::from_iter(b)
            .name(name)
            .skip_check()
            .run()
            .await?;
        Ok(())
    }

    fn as_plain(&self) -> PlainActor {
        self.plain()
    }
}

impl<T> Check for Sys<T>
where
    T: System,
    for<'a> &'a T: IntoIterator<Item = Box<&'a dyn Check>>,
{
    fn check_inputs(&self) -> std::result::Result<(), CheckError> {
        self.into_iter()
            .map(|a| a.check_inputs())
            .collect::<Result<Vec<_>, _>>()
            .map(|_| ())
    }

    fn check_outputs(&self) -> std::result::Result<(), CheckError> {
        self.into_iter()
            .map(|a| a.check_outputs())
            .collect::<Result<Vec<_>, _>>()
            .map(|_| ())
    }

    fn n_inputs(&self) -> usize {
        self.into_iter()
            .map(|a: Box<&dyn Check>| a.n_inputs())
            .sum()
    }
    fn n_outputs(&self) -> usize {
        self.into_iter()
            .map(|a: Box<&dyn Check>| a.n_outputs())
            .sum()
    }

    fn inputs_hashes(&self) -> Vec<u64> {
        self.into_iter()
            .flat_map(|a: Box<&dyn Check>| a.inputs_hashes())
            .collect()
    }

    fn outputs_hashes(&self) -> Vec<u64> {
        self.into_iter()
            .flat_map(|a: Box<&dyn Check>| a.outputs_hashes())
            .collect()
    }

    fn _as_plain(&self) -> PlainActor {
        todo!()
    }
}

pub enum New {}
pub enum Built {}

pub struct Sys<T: System, S = Built> {
    sys: T,
    state: PhantomData<S>,
}

impl<T: System, S> Clone for Sys<T, S> {
    fn clone(&self) -> Self {
        let mut sys = self.sys.clone();
        sys.build().unwrap();
        Self {
            sys,
            state: PhantomData,
        }
    }
}

impl<T: System> Deref for Sys<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.sys
    }
}

impl<T: System> DerefMut for Sys<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sys
    }
}
impl<T: System> Sys<T, New> {
    pub fn new(sys: T) -> Self {
        Self {
            sys,
            state: PhantomData,
        }
    }

    pub fn build(self) -> anyhow::Result<Sys<T>> {
        let mut this: Sys<T> = Sys {
            sys: self.sys,
            state: PhantomData,
        };
        this.sys.build()?;
        Ok(this)
    }
}
impl<T: System + SystemFlowChart> Sys<T> {
    pub fn flowchart(self) -> Self {
        self.sys.flowchart();
        self
    }
}

impl<T: System> Display for Sys<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.sys)
    }
}

pub trait SystemInput<C, const NI: usize, const NO: usize>
where
    C: interface::Update,
{
    fn input(&mut self) -> &mut Actor<C, NI, NO>;
}

pub trait SystemOutput<C, const NI: usize, const NO: usize>
where
    C: interface::Update,
{
    fn output(&mut self) -> &mut Actor<C, NI, NO>;
}

impl<
        T: System + SystemInput<C, NI, NO>,
        C: interface::Update,
        const NI: usize,
        const NO: usize,
    > SystemInput<C, NI, NO> for Sys<T>
{
    fn input(&mut self) -> &mut Actor<C, NI, NO> {
        self.sys.input()
    }
}

impl<
        T: System + SystemOutput<C, NI, NO>,
        C: interface::Update,
        const NI: usize,
        const NO: usize,
    > SystemOutput<C, NI, NO> for Sys<T>
{
    fn output(&mut self) -> &mut Actor<C, NI, NO> {
        self.sys.output()
    }
}

impl<'a, T: System> IntoIterator for &'a Sys<T>
where
    &'a T: IntoIterator<
        Item = Box<&'a dyn Check>,
        IntoIter = std::vec::IntoIter<<&'a T as IntoIterator>::Item>,
    >,
{
    type Item = Box<&'a dyn Check>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.sys.into_iter()
    }
}

impl<T: System> IntoIterator for Box<Sys<T>>
where
    Box<T>: IntoIterator<
        Item = Box<dyn Task>,
        IntoIter = std::vec::IntoIter<<Box<T> as IntoIterator>::Item>,
    >,
{
    type Item = Box<dyn Task>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let q = *self;
        let w = q.sys;
        let b = Box::new(w);
        b.into_iter()
    }
}

impl<'a, T, CO, const NI: usize, const NO: usize> AddActorOutput<'a, CO, NI, NO> for Sys<T>
where
    T: System,
    Sys<T>: SystemOutput<CO, NI, NO>,
    CO: interface::Update + 'static,
{
    fn add_output(&'a mut self) -> ActorOutput<'a, Actor<CO, NI, NO>> {
        AddActorOutput::add_output(self.output())
    }
}

impl<U, T, CI, const NI: usize, const NO: usize> AddActorInput<U, CI, NI, NO> for Sys<T>
where
    T: System,
    Sys<T>: SystemInput<CI, NI, NO>,
    U: 'static + interface::UniqueIdentifier,
    CI: interface::Read<U> + 'static,
{
    fn add_input(&mut self, rx: flume::Receiver<interface::Data<U>>, hash: u64) {
        AddActorInput::add_input(self.input(), rx, hash)
    }
}

/*

impl<M, CI, CO, const NI: usize, const NO: usize> Display for System<M, CI, CO, NI, NO>
where
    M: Gateways + Clone,
    CI: Update,
    CO: Update,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", std::any::type_name::<M>())
    }
}

/// Interface for the sub-[Model](crate::model::Model) builder
pub trait BuildSystem<M, const NI: usize = 1, const NO: usize = 1> {
    /// Builds the model by connecting all actors
    fn build(&mut self) -> anyhow::Result<()>;
} */
