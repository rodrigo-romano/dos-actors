use std::fmt::Display;

use gmt_dos_actors::{
    actor::PlainActor,
    framework::model::{Check, SystemFlowChart, Task},
    prelude::*,
    system::{System, SystemInput, SystemOutput},
};
use gmt_dos_clients::{integrator::Integrator, operator, sampler::Sampler};
use interface::UID;

#[derive(UID)]
pub enum LoFi {}

#[derive(UID)]
#[uid(port = 5001)]
pub enum IntLoFi {}

#[derive(UID)]
#[uid(port = 5002)]
pub enum ResLoFi {}

#[derive(UID)]
#[uid(port = 5003)]
pub enum IntHiFi {}

#[derive(UID)]
#[alias(name = operator::Left<LoFi>, client = operator::Operator<f64>, traits = Read)]
pub enum AddLoFi {}

#[derive(UID)]
#[alias(name = operator::Left<LoFi>, client = operator::Operator<f64>, traits = Read)]
pub enum AddResLoFi {}

const W: usize = 100;

// --- WOOFER --
#[derive(Clone)]
pub struct Woofer {
    plus: Actor<operator::Operator<f64>, 1, W>,
    int: Actor<Integrator<ResLoFi>, W, 1>,
    upsampler: Actor<Sampler<Vec<f64>, ResLoFi, AddResLoFi>, W, 1>,
}

impl Display for Woofer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " -- WOOFER --")?;
        self.plus.fmt(f)?;
        self.int.fmt(f)?;
        self.upsampler.fmt(f)?;
        writeln!(f, " -- WOOFER --")?;
        Ok(())
    }
}

impl Woofer {
    pub fn new() -> Self {
        let int: Actor<_, W, 1> = Integrator::new(1).gain(0.8).into();
        let plus: Actor<_, 1, W> = operator::Operator::new("+").into();
        let upsampler: Actor<_, W, 1> = Sampler::default().into();
        Self {
            plus,
            int,
            upsampler,
        }
    }
}

impl<'a> IntoIterator for &'a Woofer {
    type Item = Box<&'a dyn Check>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(&self.plus as &dyn Check),
            Box::new(&self.int as &dyn Check),
            Box::new(&self.upsampler as &dyn Check),
        ]
        .into_iter()
    }
}

impl IntoIterator for Box<Woofer> {
    type Item = Box<dyn Task>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(self.plus) as Box<dyn Task>,
            Box::new(self.int) as Box<dyn Task>,
            Box::new(self.upsampler) as Box<dyn Task>,
        ]
        .into_iter()
    }
}

impl System for Woofer {
    fn build(&mut self) -> anyhow::Result<&mut Self> {
        self.plus
            .add_output()
            .multiplex(2)
            .build::<ResLoFi>()
            .into_input(&mut self.int)
            .into_input(&mut self.upsampler)?;

        self.int
            .add_output()
            .bootstrap()
            .build::<operator::Right<IntLoFi>>()
            .into_input(&mut self.plus)?;
        Ok(self)
    }

    fn plain(&self) -> PlainActor {
        let mut plain = PlainActor::default();
        plain.client = "WOOFER".to_string();
        plain.inputs_rate = 1;
        plain.outputs_rate = 1;
        plain.inputs = PlainActor::from(&self.plus).inputs.map(|mut x| {
            x.remove(0);
            x
        });
        plain.outputs = PlainActor::from(&self.upsampler).outputs;
        plain.graph = self.graph();
        plain
    }

    fn name(&self) -> String {
        String::from("WOOFER")
    }
}

impl SystemInput<operator::Operator<f64>, 1, W> for Woofer {
    fn input(&mut self) -> &mut Actor<operator::Operator<f64>, 1, W> {
        &mut self.plus
    }
}

impl SystemOutput<Sampler<Vec<f64>, ResLoFi, AddResLoFi>, W, 1> for Woofer {
    fn output(&mut self) -> &mut Actor<Sampler<Vec<f64>, ResLoFi, AddResLoFi>, W, 1> {
        &mut self.upsampler
    }
}
