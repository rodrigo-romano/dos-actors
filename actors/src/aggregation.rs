//! # Model aggregations
//!
//! Algebraic rules to add [Model], [SubSystem] and [Actor] to create a new model

use std::ops::{Add, AddAssign};

use interface::Update;

use crate::{
    actor::Actor,
    model,
    model::{Model, Unknown},
    subsystem::{Gateways, SubSystem},
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
    C: Update + Send + Sync + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Actor<C, NI, NO>) -> Self::Output {
        self + model!(rhs)
    }
}
/// Aggregation of a model and a subsystem into a new model
impl<M> Add<SubSystem<M>> for Model<Unknown>
where
    M: Gateways + 'static,
    <M as Gateways>::DataType: Send + Sync,
    Model<model::Unknown>: From<M>,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: SubSystem<M>) -> Self::Output {
        self + <SubSystem<M> as Into<Model<Unknown>>>::into(rhs)
    }
}

/// Aggregation of an actor and a model into a new model
impl<C, const NI: usize, const NO: usize> Add<Model<Unknown>> for Actor<C, NI, NO>
where
    C: Update + Send + Sync + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Model<Unknown>) -> Self::Output {
        model!(self) + rhs
    }
}
/// Aggregation of a subsystem and a model into a new model
impl<M> Add<Model<Unknown>> for SubSystem<M>
where
    M: Gateways + 'static,
    <M as Gateways>::DataType: Send + Sync,
    Model<model::Unknown>: From<M>,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Model<Unknown>) -> Self::Output {
        rhs + <SubSystem<M> as Into<Model<Unknown>>>::into(self)
    }
}

/// Aggregation of actors into a model
impl<A, const A_NI: usize, const A_NO: usize, B, const B_NI: usize, const B_NO: usize>
    Add<Actor<B, B_NI, B_NO>> for Actor<A, A_NI, A_NO>
where
    A: Update + Send + Sync + 'static,
    B: Update + Send + Sync + 'static,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Actor<B, B_NI, B_NO>) -> Self::Output {
        model!(self) + model!(rhs)
    }
}
/// Aggregation of subsystems into a model
impl<Right, Left> Add<SubSystem<Right>> for SubSystem<Left>
where
    Right: Gateways + 'static,
    <Right as Gateways>::DataType: Send + Sync,
    Model<model::Unknown>: From<Right>,
    Left: Gateways + 'static,
    <Left as Gateways>::DataType: Send + Sync,
    Model<model::Unknown>: From<Left>,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: SubSystem<Right>) -> Self::Output {
        <SubSystem<Left> as Into<Model<Unknown>>>::into(self)
            + <SubSystem<Right> as Into<Model<Unknown>>>::into(rhs)
    }
}

/// Aggregation of an actor and a subsystem into a new model
impl<M, C, const NI: usize, const NO: usize> Add<SubSystem<M>> for Actor<C, NI, NO>
where
    C: Update + Send + Sync + 'static,
    M: Gateways + 'static,
    <M as Gateways>::DataType: Send + Sync,
    Model<model::Unknown>: From<M>,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: SubSystem<M>) -> Self::Output {
        model!(self) + rhs
    }
}
/// Aggregation of an subsystem and an actor into a new model
impl<M, C, const NI: usize, const NO: usize> Add<Actor<C, NI, NO>> for SubSystem<M>
where
    C: Update + Send + Sync + 'static,
    M: Gateways + 'static,
    <M as Gateways>::DataType: Send + Sync,
    Model<model::Unknown>: From<M>,
{
    type Output = Model<Unknown>;

    fn add(self, rhs: Actor<C, NI, NO>) -> Self::Output {
        self + model!(rhs)
    }
}

impl<C, const NI: usize, const NO: usize> AddAssign<Actor<C, NI, NO>> for Model<Unknown>
where
    C: Update + Send + Sync + 'static,
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
