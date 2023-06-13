use crate::{model, Actor, Update};

use super::{Actors, Model, ModelError, Ready, Result, Unknown};
use std::{
    marker::PhantomData,
    ops::{Add, AddAssign},
    time::Instant,
};

impl Default for Model<Unknown> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            actors: Default::default(),
            task_handles: Default::default(),
            state: Default::default(),
            start: Instant::now(),
            verbose: true,
        }
    }
}

impl Model<Unknown> {
    /// Returns a new model
    pub fn new(actors: Actors) -> Self {
        Self {
            name: None,
            actors: Some(actors),
            task_handles: None,
            state: PhantomData,
            start: Instant::now(),
            verbose: true,
        }
    }
    /// Sets the model name
    pub fn name<S: Into<String>>(self, name: S) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }
    /// Quiet mode
    pub fn quiet(mut self) -> Self {
        self.verbose = false;
        self
    }
    /// Validates actors inputs and outputs
    pub fn check(self) -> Result<Model<Ready>> {
        let (n_inputs, n_outputs) = self.n_io();
        assert_eq!(
            n_inputs, n_outputs,
            "I/O #({},{}) don't match, did you forget to add some actors to the model?",
            n_inputs, n_outputs
        );
        match self.actors {
            Some(ref actors) => {
                let mut inputs_hashes = vec![];
                let mut outputs_hashes = vec![];
                for actor in actors {
                    actor.check_inputs()?;
                    actor.check_outputs()?;
                    inputs_hashes.append(&mut actor.inputs_hashes());
                    outputs_hashes.append(&mut actor.outputs_hashes());
                }
                let hashes_diff = outputs_hashes
                    .into_iter()
                    .zip(inputs_hashes.into_iter())
                    .map(|(o, i)| o as i128 - i as i128)
                    .sum::<i128>();
                assert_eq!(hashes_diff,0i128,
                "I/O hashes difference: expected 0, found {}, did you forget to add some actors to the model?",
                hashes_diff);
                Ok(Model::<Ready> {
                    name: self.name,
                    actors: self.actors,
                    task_handles: None,
                    state: PhantomData,
                    start: Instant::now(),
                    verbose: self.verbose,
                })
            }
            None => Err(ModelError::NoActors),
        }
    }
}

/// Aggregation of models into a new model
impl Add for Model<Unknown> {
    type Output = Model<Unknown>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.actors, rhs.actors) {
            (None, None) => Model::new(vec![]),
            (None, Some(b)) => Model::new(b),
            (Some(a), None) => Model::new(a),
            (Some(mut a), Some(mut b)) => {
                a.append(&mut b);
                Model::new(a)
            }
        }
    }
}

/// Aggregation of a model and an actor into a new model
impl<C, const NI: usize, const NO: usize> Add<Actor<C, NI, NO>> for Model<Unknown>
where
    C: Update + Send + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Actor<C, NI, NO>) -> Self::Output {
        self + model!(rhs)
    }
}

/// Aggregation of an actor and a model into a new model
impl<C, const NI: usize, const NO: usize> Add<Model<Unknown>> for Actor<C, NI, NO>
where
    C: Update + Send + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Model<Unknown>) -> Self::Output {
        model!(self) + rhs
    }
}

/// Aggregation of actors into a model
impl<A, const A_NI: usize, const A_NO: usize, B, const B_NI: usize, const B_NO: usize>
    Add<Actor<B, B_NI, B_NO>> for Actor<A, A_NI, A_NO>
where
    A: Update + Send + 'static,
    B: Update + Send + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Actor<B, B_NI, B_NO>) -> Self::Output {
        model!(self) + model!(rhs)
    }
}

impl<C, const NI: usize, const NO: usize> AddAssign<Actor<C, NI, NO>> for Model<Unknown>
where
    C: Update + Send + 'static,
{
    fn add_assign(&mut self, rhs: Actor<C, NI, NO>) {
        self.actors.get_or_insert(vec![]).push(Box::new(rhs));
    }
}

impl AddAssign<Model<Unknown>> for Model<Unknown> {
    fn add_assign(&mut self, mut rhs: Model<Unknown>) {
        if let Some(actors) = rhs.actors.as_mut() {
            self.actors.get_or_insert(vec![]).append(actors);
        }
    }
}
