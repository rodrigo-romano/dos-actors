mod m1_lom;
mod m2_lom;
mod m2rb_lom;

use std::fmt::Display;

use gmt_dos_actors::{
    actor::{Actor, PlainActor, Terminator},
    framework::{
        model::{Check, SystemFlowChart, Task},
        network::AddActorOutput,
    },
    prelude::{AddOuput, TryIntoInputs},
    system::{System, SystemInput},
};
use gmt_dos_clients_io::optics::SegmentPiston;
use gmt_dos_clients_lom::LinearOpticalModel;
use gmt_dos_clients_scope::server::{Monitor, Scope, XScope};
use io::{M1SegmentPiston, M2RBSegmentPiston, M2SegmentPiston};
pub use m1_lom::M1Lom;
pub use m2_lom::M2Lom;
pub use m2rb_lom::M2RBLom;

#[derive(Debug, Clone)]
pub struct Scopes {
    m1_lom: Actor<M1Lom>,
    m2_lom: Actor<M2Lom>,
    m2rb_lom: Actor<M2RBLom>,
    lom: Actor<LinearOpticalModel>,
    segment_piston_scope: Terminator<XScope<SegmentPiston<-9>>>,
    m1_segment_piston_scope: Terminator<XScope<M1SegmentPiston>>,
    m2_segment_piston_scope: Terminator<XScope<M2SegmentPiston>>,
    m2rb_segment_piston_scope: Terminator<XScope<M2RBSegmentPiston>>,
}

impl Scopes {
    pub fn new(sim_sampling_frequency: f64, monitor: &mut Monitor) -> anyhow::Result<Self> {
        let lom = LinearOpticalModel::new()?;
        Ok(Self {
            m1_lom: M1Lom::from(lom.clone()).into(),
            m2_lom: M2Lom::from(lom.clone()).into(),
            m2rb_lom: M2RBLom::from(lom.clone()).into(),
            lom: lom.into(),
            segment_piston_scope: Scope::<SegmentPiston<-9>>::builder(monitor)
                .sampling_frequency(sim_sampling_frequency as f64)
                .build()?
                .into(),
            m1_segment_piston_scope: Scope::<M1SegmentPiston>::builder(monitor)
                .sampling_frequency(sim_sampling_frequency as f64)
                .build()?
                .into(),
            m2_segment_piston_scope: Scope::<M2SegmentPiston>::builder(monitor)
                .sampling_frequency(sim_sampling_frequency as f64)
                .build()?
                .into(),
            m2rb_segment_piston_scope: Scope::<M2RBSegmentPiston>::builder(monitor)
                .sampling_frequency(sim_sampling_frequency as f64)
                .build()?
                .into(),
        })
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Edge Sensors Integrated Model Scopes")
    }
}

impl System for Scopes {
    fn build(&mut self) -> anyhow::Result<&mut Self> {
        self.m1_lom
            .add_output()
            .build::<M1SegmentPiston>()
            .into_input(&mut self.m1_segment_piston_scope)?;
        self.m2_lom
            .add_output()
            .build::<M2SegmentPiston>()
            .into_input(&mut self.m2_segment_piston_scope)?;
        self.m2rb_lom
            .add_output()
            .build::<M2RBSegmentPiston>()
            .into_input(&mut self.m2rb_segment_piston_scope)?;
        self.lom
            .add_output()
            .build::<SegmentPiston<-9>>()
            .into_input(&mut self.segment_piston_scope)?;
        Ok(self)
    }

    fn plain(&self) -> gmt_dos_actors::actor::PlainActor {
        let mut plain = PlainActor::default();
        plain.client = self.name();
        plain.inputs_rate = 1;
        plain.outputs_rate = 0;
        plain.inputs = Some(
            PlainActor::from(&self.m1_lom)
                .inputs
                .unwrap()
                .into_iter()
                .chain(PlainActor::from(&self.m2_lom).inputs.unwrap().into_iter())
                .chain(PlainActor::from(&self.m2rb_lom).inputs.unwrap().into_iter())
                .chain(PlainActor::from(&self.lom).inputs.unwrap().into_iter())
                .collect::<Vec<_>>(),
        );
        plain.graph = self.graph();
        plain
    }

    fn name(&self) -> String {
        String::from("Edge Sensors Integrated Model Scopes")
    }
}

impl<'a> IntoIterator for &'a Scopes {
    type Item = Box<&'a dyn Check>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(&self.m1_lom as &dyn Check),
            Box::new(&self.m2_lom as &dyn Check),
            Box::new(&self.m2rb_lom as &dyn Check),
            Box::new(&self.lom as &dyn Check),
            Box::new(&self.m1_segment_piston_scope as &dyn Check),
            Box::new(&self.m2_segment_piston_scope as &dyn Check),
            Box::new(&self.m2rb_segment_piston_scope as &dyn Check),
            Box::new(&self.segment_piston_scope as &dyn Check),
        ]
        .into_iter()
    }
}

impl IntoIterator for Box<Scopes> {
    type Item = Box<dyn Task>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            Box::new(self.m1_lom) as Box<dyn Task>,
            Box::new(self.m2_lom) as Box<dyn Task>,
            Box::new(self.m2rb_lom) as Box<dyn Task>,
            Box::new(self.lom) as Box<dyn Task>,
            Box::new(self.m1_segment_piston_scope) as Box<dyn Task>,
            Box::new(self.m2_segment_piston_scope) as Box<dyn Task>,
            Box::new(self.m2rb_segment_piston_scope) as Box<dyn Task>,
            Box::new(self.segment_piston_scope) as Box<dyn Task>,
        ]
        .into_iter()
    }
}

impl SystemInput<M1Lom, 1, 1> for Scopes {
    fn input(&mut self) -> &mut Actor<M1Lom, 1, 1> {
        &mut self.m1_lom
    }
}

impl SystemInput<M2Lom, 1, 1> for Scopes {
    fn input(&mut self) -> &mut Actor<M2Lom, 1, 1> {
        &mut self.m2_lom
    }
}

impl SystemInput<M2RBLom, 1, 1> for Scopes {
    fn input(&mut self) -> &mut Actor<M2RBLom, 1, 1> {
        &mut self.m2rb_lom
    }
}

impl SystemInput<LinearOpticalModel, 1, 1> for Scopes {
    fn input(&mut self) -> &mut Actor<LinearOpticalModel, 1, 1> {
        &mut self.lom
    }
}
