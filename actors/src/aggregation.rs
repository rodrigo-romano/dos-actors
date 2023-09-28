//! # Model aggregations
//!
//! Algebraic rules to add [Model], [SubSystem] and [Actor] to create a new model

use std::ops::{Add, AddAssign};

use interface::Update;

use crate::{
    actor::Actor,
    model,
    model::{Model, Unknown},
    subsystem::{BuildSystem, Gateways, GetField, SubSystem},
};

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
    C: Update + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Actor<C, NI, NO>) -> Self::Output {
        self + model!(rhs)
    }
}
/// Aggregation of a model and a subsystem into a new model
impl<M> Add<SubSystem<M>> for Model<Unknown>
where
    M: Gateways + BuildSystem<M> + GetField + 'static,
    Model<model::Unknown>: From<M>,
{
    type Output = Model<Unknown>;

    fn add(mut self, rhs: SubSystem<M>) -> Self::Output {
        self.actors
            .as_mut()
            .map(|actors| actors.push(Box::new(rhs)));
        self
    }
}

/// Aggregation of an actor and a model into a new model
impl<C, const NI: usize, const NO: usize> Add<Model<Unknown>> for Actor<C, NI, NO>
where
    C: Update + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Model<Unknown>) -> Self::Output {
        model!(self) + rhs
    }
}
/// Aggregation of a subsystem and a model into a new model
impl<M> Add<Model<Unknown>> for SubSystem<M>
where
    M: Gateways + BuildSystem<M> + GetField + 'static,
    Model<model::Unknown>: From<M>,
{
    type Output = Model<Unknown>;

    fn add(self, mut rhs: Model<Unknown>) -> Self::Output {
        rhs.actors
            .as_mut()
            .map(|actors| actors.push(Box::new(self)));
        rhs
    }
}

/// Aggregation of actors into a model
impl<A, const A_NI: usize, const A_NO: usize, B, const B_NI: usize, const B_NO: usize>
    Add<Actor<B, B_NI, B_NO>> for Actor<A, A_NI, A_NO>
where
    A: Update + 'static,
    B: Update + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Actor<B, B_NI, B_NO>) -> Self::Output {
        model!(self) + model!(rhs)
    }
}
/// Aggregation of subsystems into a model
impl<Right, Left> Add<SubSystem<Right>> for SubSystem<Left>
where
    Right: Gateways + BuildSystem<Right> + GetField + 'static,
    Model<model::Unknown>: From<Right>,
    Left: Gateways + BuildSystem<Left> + GetField + 'static,
    Model<model::Unknown>: From<Left>,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: SubSystem<Right>) -> Self::Output {
        model!(self, rhs)
    }
}

/// Aggregation of an actor and a subsystem into a new model
impl<M, C, const NI: usize, const NO: usize> Add<SubSystem<M>> for Actor<C, NI, NO>
where
    C: Update + 'static,
    M: Gateways + BuildSystem<M> + GetField + 'static,
    Model<model::Unknown>: From<M>,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: SubSystem<M>) -> Self::Output {
        model!(self, rhs)
    }
}
/// Aggregation of an subsystem and an actor into a new model
impl<M, C, const NI: usize, const NO: usize> Add<Actor<C, NI, NO>> for SubSystem<M>
where
    C: Update + 'static,
    M: Gateways + BuildSystem<M> + GetField + 'static,
    Model<model::Unknown>: From<M>,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Actor<C, NI, NO>) -> Self::Output {
        model!(self, rhs)
    }
}

impl<C, const NI: usize, const NO: usize> AddAssign<Actor<C, NI, NO>> for Model<Unknown>
where
    C: Update + 'static,
{
    fn add_assign(&mut self, rhs: Actor<C, NI, NO>) {
        self.actors.get_or_insert(vec![]).push(Box::new(rhs));
    }
}

impl<M, const NI: usize, const NO: usize> AddAssign<SubSystem<M, NI, NO>> for Model<Unknown>
where
    M: Gateways + GetField + BuildSystem<M, NI, NO> + 'static,
    Model<Unknown>: From<M>,
{
    fn add_assign(&mut self, rhs: SubSystem<M, NI, NO>) {
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
