use std::fmt::Display;

use gmt_dos_actors::{
    actor::PlainActor,
    framework::model::{Check, SystemFlowChart, Task},
    prelude::*,
    system::{System, SystemInput, SystemOutput},
};
use gmt_dos_clients::{operator, Integrator};
use interface::UID;

#[derive(UID)]
#[uid(port = 5004)]
pub enum ResHiFi {}

#[derive(UID)]
#[uid(port = 5003)]
pub enum IntHiFi {}

#[derive(Clone)]
pub struct Tweeter {
    plus: Actor<operator::Operator<f64>>,
    int: Actor<Integrator<ResHiFi>>,
}

impl Display for Tweeter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " -- TWEETER --")?;
        self.plus.fmt(f)?;
        self.int.fmt(f)?;
        writeln!(f, " -- TWEETER --")?;
        Ok(())
    }
}

impl Tweeter {
    pub fn new() -> Self {
        let int: Actor<_> = Integrator::new(1).gain(0.1).into();
        let plus: Actor<_> = operator::Operator::new("+").into();
        Self { plus, int }
    }
}

impl<'a> IntoIterator for &'a Tweeter {
    type Item = Box<&'a dyn Check>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(&self.plus as &dyn Check),
            Box::new(&self.int as &dyn Check),
        ]
        .into_iter()
    }
}

impl IntoIterator for Box<Tweeter> {
    type Item = Box<dyn Task>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(self.plus) as Box<dyn Task>,
            Box::new(self.int) as Box<dyn Task>,
        ]
        .into_iter()
    }
}

impl System for Tweeter {
    fn build(&mut self) -> anyhow::Result<&mut Self> {
        self.plus
            .add_output()
            .build::<ResHiFi>()
            .into_input(&mut self.int)?;

        self.int
            .add_output()
            .bootstrap()
            .build::<operator::Right<IntHiFi>>()
            .into_input(&mut self.plus)?;
        Ok(self)
    }

    fn plain(&self) -> PlainActor {
        let mut plain = PlainActor::default();
        plain.client = self.name();
        plain.inputs_rate = 1;
        plain.outputs_rate = 1;
        plain.inputs = PlainActor::from(&self.plus).inputs.map(|mut x| {
            x.remove(0);
            x
        });
        plain.outputs = PlainActor::from(&self.plus).outputs;
        plain.graph = self.graph();
        plain
    }

    fn name(&self) -> String {
        String::from("TWEETER")
    }
}

impl SystemInput<operator::Operator<f64>, 1, 1> for Tweeter {
    fn input(&mut self) -> &mut Actor<operator::Operator<f64>, 1, 1> {
        &mut self.plus
    }
}

impl SystemOutput<operator::Operator<f64>, 1, 1> for Tweeter {
    fn output(&mut self) -> &mut Actor<operator::Operator<f64>, 1, 1> {
        &mut self.plus
    }
}
