use std::fmt::Display;

use crate::{subsystems::SegmentControl, Actuators, Hardpoints, LoadCells};
use gmt_dos_actors::{
    actor::{Actor, PlainActor},
    framework::network::AddActorOutput,
    prelude::TryIntoInputs,
    system::{System, SystemInput, SystemOutput},
    Check,
};
use gmt_dos_actors::{prelude::AddOuput, Task};
use gmt_dos_clients::Sampler;
use gmt_dos_clients_io::gmt_m1::segment::{
    ActuatorCommandForces, BarycentricForce, HardpointsForces,
};

impl<const S: u8, const R: usize> System for SegmentControl<S, R> {
    fn name(&self) -> String {
        format!("M1S{S}")
    }

    fn build(&mut self) -> anyhow::Result<&mut Self> {
        self.sampler
            .add_output()
            .build::<ActuatorCommandForces<S>>()
            .into_input(&mut self.actuators)?;

        self.hardpoints
            .add_output()
            .build::<HardpointsForces<S>>()
            .into_input(&mut self.loadcells)?;

        self.loadcells
            .add_output()
            .build::<BarycentricForce<S>>()
            .into_input(&mut self.actuators)?;
        Ok(self)
    }

    fn plain(&self) -> gmt_dos_actors::actor::PlainActor {
        let mut plain = PlainActor::default();
        plain.client = self.name();
        plain.inputs_rate = 1;
        plain.outputs_rate = 1;
        let iter = PlainActor::from(&self.hardpoints)
            .inputs
            .unwrap()
            .into_iter()
            .chain(PlainActor::from(&self.sampler).inputs.unwrap().into_iter())
            .chain(
                PlainActor::from(&self.loadcells)
                    .inputs
                    .unwrap()
                    .into_iter(),
            );
        plain.inputs = Some(iter.collect());
        let iter = PlainActor::from(&self.hardpoints)
            .outputs
            .unwrap()
            .into_iter()
            .chain(
                PlainActor::from(&self.actuators)
                    .outputs
                    .unwrap()
                    .into_iter(),
            );
        plain.outputs = Some(iter.collect());
        plain
    }
}

impl<'a, const S: u8, const R: usize> IntoIterator for &'a SegmentControl<S, R> {
    type Item = Box<&'a dyn Check>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(&self.sampler as &dyn Check),
            Box::new(&self.hardpoints as &dyn Check),
            Box::new(&self.actuators as &dyn Check),
            Box::new(&self.loadcells as &dyn Check),
        ]
        .into_iter()
    }
}

impl<const S: u8, const R: usize> IntoIterator for Box<SegmentControl<S, R>> {
    type Item = Box<dyn Task>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(self.sampler) as Box<dyn Task>,
            Box::new(self.hardpoints) as Box<dyn Task>,
            Box::new(self.actuators) as Box<dyn Task>,
            Box::new(self.loadcells) as Box<dyn Task>,
        ]
        .into_iter()
    }
}

impl<const S: u8, const R: usize> SystemInput<Hardpoints, 1, 1> for SegmentControl<S, R> {
    fn input(&mut self) -> &mut Actor<Hardpoints, 1, 1> {
        &mut self.hardpoints
    }
}

impl<const S: u8, const R: usize> SystemInput<Sampler<Vec<f64>, ActuatorCommandForces<S>>, 1, R>
    for SegmentControl<S, R>
{
    fn input(&mut self) -> &mut Actor<Sampler<Vec<f64>, ActuatorCommandForces<S>>, 1, R> {
        &mut self.sampler
    }
}

impl<const S: u8, const R: usize> SystemInput<LoadCells, 1, R> for SegmentControl<S, R> {
    fn input(&mut self) -> &mut Actor<LoadCells, 1, R> {
        &mut self.loadcells
    }
}

impl<const S: u8, const R: usize> SystemOutput<Hardpoints, 1, 1> for SegmentControl<S, R> {
    fn output(&mut self) -> &mut Actor<Hardpoints, 1, 1> {
        &mut self.hardpoints
    }
}

impl<const S: u8, const R: usize> SystemOutput<Actuators<S>, R, 1> for SegmentControl<S, R> {
    fn output(&mut self) -> &mut Actor<Actuators<S>, R, 1> {
        &mut self.actuators
    }
}

/* impl<const S: u8, const R: usize> AddActorInput<RBM<S>, Hardpoints, 1> for SegmentControl<S, R> {
    fn add_input(&mut self, rx: flume::Receiver<interface::Data<RBM<S>>>, hash: u64) {
        AddActorInput::add_input(&mut self.hardpoints, rx, hash)
    }
}

impl<const S: u8, const R: usize>
    AddActorInput<ActuatorCommandForces<S>, Sampler<Vec<f64>, ActuatorCommandForces<S>>, 1>
    for SegmentControl<S, R>
{
    fn add_input(
        &mut self,
        rx: flume::Receiver<interface::Data<ActuatorCommandForces<S>>>,
        hash: u64,
    ) {
        AddActorInput::add_input(&mut self.sampler, rx, hash)
    }
}

impl<const S: u8, const R: usize> AddActorInput<HardpointsMotion<S>, LoadCells, 1>
    for SegmentControl<S, R>
{
    fn add_input(&mut self, rx: flume::Receiver<interface::Data<HardpointsMotion<S>>>, hash: u64) {
        AddActorInput::add_input(&mut self.loadcells, rx, hash)
    }
}

impl<'a, const S: u8, const R: usize> AddActorOutput<'a, Hardpoints, 1, 1>
    for SegmentControl<S, R>
{
    fn add_output(&'a mut self) -> ActorOutput<'a, Actor<Hardpoints, 1, 1>>
    where
        ActorOutput<'a, Actor<Hardpoints, 1, 1>>: AddOuput<'a, Hardpoints, 1, 1>,
    {
        AddActorOutput::add_output(&mut self.hardpoints)
    }
}

impl<'a, const S: u8, const R: usize> AddActorOutput<'a, Actuators<S>, R, 1>
    for SegmentControl<S, R>
{
    fn add_output(&'a mut self) -> ActorOutput<'a, Actor<Actuators<S>, R, 1>> {
        AddActorOutput::add_output(&mut self.actuators)
    }
} */

/* impl<const S: u8, const R: usize> Check for SegmentControl<S, R> {
    fn check_inputs(
        &self,
    ) -> std::result::Result<(), gmt_dos_actors::framework::model::CheckError> {
        self.into_iter()
            .map(|a| a.check_inputs())
            .collect::<Result<Vec<_>, _>>()
            .map(|_| ())
    }

    fn check_outputs(
        &self,
    ) -> std::result::Result<(), gmt_dos_actors::framework::model::CheckError> {
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
        let mut plain = PlainActor::default();
        plain.client = self.name();
        plain.inputs_rate = 1;
        let mut inputs = self.hardpoints._as_plain().inputs.unwrap();
        inputs.append(self.actuators._as_plain().inputs.as_mut().unwrap());
        plain.inputs = Some(inputs);
        plain.outputs_rate = 1;
        let mut outputs = self.hardpoints._as_plain().outputs.unwrap();
        outputs.append(self.actuators._as_plain().outputs.as_mut().unwrap());
        plain.outputs = self.hardpoints._as_plain().outputs;
        plain
    }
}
 */

impl<const S: u8, const R: usize> Display for SegmentControl<S, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/* #[async_trait::async_trait]
impl<const S: u8, const R: usize> Task for SegmentControl<S, R> {
    async fn async_run(&mut self) -> std::result::Result<(), TaskError> {
        todo!()
    }

    async fn task(mut self: Box<Self>) -> std::result::Result<(), TaskError> {
        Model::<Unknown>::from_iter(self).skip_check().run().await?;
        Ok(())
    }

    fn as_plain(&self) -> PlainActor {
        todo!()
    }
}
 */
