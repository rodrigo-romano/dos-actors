use std::fmt::Display;

use gmt_dos_actors::{
    actor::{Actor, PlainActor},
    framework::{
        model::{Check, SystemFlowChart, Task},
        network::AddActorOutput,
    },
    prelude::{AddOuput, TryIntoInputs},
    system::{System, SystemInput, SystemOutput},
};
use gmt_dos_clients::{
    integrator::Integrator,
    operator::{Left, Operator},
};
use gmt_dos_clients_io::gmt_m1::M1EdgeSensors;

#[derive(Debug, Clone)]
pub struct M1EdgeSensorsToRbm {
    adder: Actor<Operator<f64>>,
    control: Actor<Integrator<M1EdgeSensors>>,
}

impl M1EdgeSensorsToRbm {
    pub fn new() -> Self {
        Self {
            adder: (Operator::new("+"), "+").into(),
            control: Integrator::new(42).gain(1e-3).into(),
        }
    }
}

impl Display for M1EdgeSensorsToRbm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Closed-loop M1 Edge Sensors To RBM")
    }
}

impl System for M1EdgeSensorsToRbm {
    fn build(&mut self) -> anyhow::Result<&mut Self> {
        self.control
            .add_output()
            .build::<Left<M1EdgeSensors>>()
            .into_input(&mut self.adder)?;
        Ok(self)
    }

    fn plain(&self) -> gmt_dos_actors::actor::PlainActor {
        let mut plain = PlainActor::default();
        plain.client = self.name();
        plain.inputs_rate = 1;
        plain.outputs_rate = 1;
        plain.inputs = match (
            PlainActor::from(&self.control).inputs,
            PlainActor::from(&self.adder).inputs.map(|input| {
                input
                    .into_iter()
                    .filter(|input| input.filter(|x| x.name.contains("Right")))
                    .collect::<Vec<_>>()
            }),
        ) {
            (Some(mut control), Some(adder)) => {
                control.extend(adder);
                Some(control)
            }
            _ => None,
        };
        plain.outputs = PlainActor::from(&self.adder).outputs;
        plain.graph = self.graph();
        plain
    }

    fn name(&self) -> String {
        String::from("Closed-loop M1 Edge Sensors To RBM")
    }
}

impl<'a> IntoIterator for &'a M1EdgeSensorsToRbm {
    type Item = Box<&'a dyn Check>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(&self.adder as &dyn Check),
            Box::new(&self.control as &dyn Check),
        ]
        .into_iter()
    }
}

impl IntoIterator for Box<M1EdgeSensorsToRbm> {
    type Item = Box<dyn Task>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(self.adder) as Box<dyn Task>,
            Box::new(self.control) as Box<dyn Task>,
        ]
        .into_iter()
    }
}

impl SystemInput<Integrator<M1EdgeSensors>, 1, 1> for M1EdgeSensorsToRbm {
    fn input(&mut self) -> &mut Actor<Integrator<M1EdgeSensors>, 1, 1> {
        &mut self.control
    }
}

impl SystemInput<Operator<f64>, 1, 1> for M1EdgeSensorsToRbm {
    fn input(&mut self) -> &mut Actor<Operator<f64>, 1, 1> {
        &mut self.adder
    }
}

impl SystemOutput<Operator<f64>, 1, 1> for M1EdgeSensorsToRbm {
    fn output(&mut self) -> &mut Actor<Operator<f64>, 1, 1> {
        &mut self.adder
    }
}
