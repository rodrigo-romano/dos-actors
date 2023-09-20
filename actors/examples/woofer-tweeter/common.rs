use gmt_dos_actors::{
    model::{subsystem::BuildSystem, Unknown},
    prelude::*,
};
use gmt_dos_clients::{operator, Integrator, Sampler};
use interface::{
    gateway::{self, Gateways, WayIn, WayOut},
    UID,
};

#[derive(UID)]
pub enum LoFi {}

#[derive(UID)]
pub enum IntLoFi {}

#[derive(UID)]
pub enum ResLoFi {}

#[derive(UID)]
pub enum IntHiFi {}

#[derive(UID)]
pub enum ResHiFi {}

const W: usize = 100;

pub struct Woofer {
    plus: Actor<operator::Operator<f64>, 1, W>,
    int: Actor<Integrator<ResLoFi>, W, 1>,
    upsampler: Actor<Sampler<Vec<f64>, ResLoFi>, W, 1>,
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

impl Gateways for Woofer {
    type DataType = Vec<f64>;
}

impl BuildSystem<Woofer> for Woofer {
    fn build(
        &mut self,
        gateway_in: &mut Actor<WayIn<Woofer>>,
        gateway_out: &mut Actor<WayOut<Woofer>>,
    ) -> anyhow::Result<()> {
        gateway_in
            .add_output()
            .build::<AddLoFi>()
            .into_input(&mut self.plus)?;

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

        self.upsampler
            .add_output()
            .build::<ResLoFi>()
            .into_input(gateway_out)?;

        Ok(())
    }
}

impl From<Woofer> for Model<Unknown> {
    fn from(w: Woofer) -> Self {
        model![w.int, w.plus, w.upsampler]
    }
}

#[derive(UID)]
#[alias(name = operator::Left<LoFi>, client = operator::Operator<f64>, traits = Read)]
pub enum AddLoFi {}

impl gateway::In for AddLoFi {}
impl gateway::Out for ResLoFi {}

pub struct Tweeter {
    plus: Actor<operator::Operator<f64>>,
    int: Actor<Integrator<ResHiFi>>,
}
impl Gateways for Tweeter {
    type DataType = Vec<f64>;
}

impl Tweeter {
    pub fn new() -> Self {
        let int: Actor<_> = Integrator::new(1).gain(0.1).into();
        let plus: Actor<_> = operator::Operator::new("+").into();
        Self { plus, int }
    }
}

impl BuildSystem<Tweeter> for Tweeter {
    fn build(
        &mut self,
        gateway_in: &mut Actor<WayIn<Tweeter>>,
        gateway_out: &mut Actor<WayOut<Tweeter>>,
    ) -> anyhow::Result<()> {
        gateway_in
            .add_output()
            .build::<AddResLoFi>()
            .into_input(&mut self.plus)?;

        self.plus
            .add_output()
            .multiplex(2)
            .build::<ResHiFi>()
            .into_input(&mut self.int)
            .into_input(gateway_out)?;

        self.int
            .add_output()
            .bootstrap()
            .build::<operator::Right<IntHiFi>>()
            .into_input(&mut self.plus)?;

        Ok(())
    }
}

#[derive(UID)]
#[alias(name = operator::Left<LoFi>, client = operator::Operator<f64>, traits = Read)]
pub enum AddResLoFi {}

impl gateway::In for ResLoFi {}
impl gateway::In for AddResLoFi {}
impl gateway::Out for ResHiFi {}

impl From<Tweeter> for Model<Unknown> {
    fn from(t: Tweeter) -> Self {
        model![t.int, t.plus]
    }
}
