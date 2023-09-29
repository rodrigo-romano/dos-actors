use gmt_dos_actors::{
    framework::model::Check,
    prelude::*,
    subsystem::{
        gateway::{self, Gateways, WayIn, WayOut},
        BuildSystem, GetField,
    },
};
use gmt_dos_clients::{operator, Integrator, Sampler};
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
#[uid(port = 5004)]
pub enum ResHiFi {}

const W: usize = 100;

// --- WOOFER --

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

impl GetField for Woofer {
    fn get_field(&self, idx: usize) -> Option<&dyn Check> {
        match idx {
            0 => Some(&self.plus as &dyn Check),
            1 => Some(&self.int as &dyn Check),
            2 => Some(&self.upsampler as &dyn Check),
            _ => None,
        }
    }
}

/* impl Check for Woofer {
    fn check_inputs(&self) -> std::result::Result<(), gmt_dos_actors::CheckError> {
        self.plus.check_inputs()?;
        self.int.check_inputs()?;
        self.upsampler.check_inputs()?;
        Ok(())
    }

    fn check_outputs(&self) -> std::result::Result<(), gmt_dos_actors::CheckError> {
        self.plus.check_outputs()?;
        self.int.check_outputs()?;
        self.upsampler.check_outputs()?;
        Ok(())
    }

    fn n_inputs(&self) -> usize {
        self.plus.n_inputs() + self.int.n_inputs() + self.upsampler.n_inputs()
    }

    fn n_outputs(&self) -> usize {
        self.plus.n_outputs() + self.int.n_outputs() + self.upsampler.n_outputs()
    }

    fn inputs_hashes(&self) -> Vec<u64> {
        vec![
            self.plus.inputs_hashes(),
            self.int.inputs_hashes(),
            self.upsampler.inputs_hashes(),
        ]
    }

    fn outputs_hashes(&self) -> Vec<u64> {
        vec![
            self.plus.outputs_hashes(),
            self.int.outputs_hashes(),
            self.upsampler.outputs_hashes(),
        ]
    }
}
 */
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

// --- TWEETER ---

pub struct Tweeter {
    plus: Actor<operator::Operator<f64>>,
    int: Actor<Integrator<ResHiFi>>,
}

impl GetField for Tweeter {
    fn get_field(&self, idx: usize) -> Option<&dyn Check> {
        match idx {
            0 => Some(&self.plus as &dyn Check),
            1 => Some(&self.int as &dyn Check),
            _ => None,
        }
    }
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
