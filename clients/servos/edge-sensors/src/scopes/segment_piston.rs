mod m1_lom;
mod m2_lom;
mod m2rb_lom;

use std::{fmt::Display, sync::Arc};

use gmt_dos_actors::{
    actor::{Actor, PlainActor, Terminator},
    framework::{
        model::{Check, FlowChart, Task},
        network::AddActorOutput,
    },
    prelude::{AddOuput, TryIntoInputs},
    system::{System, SystemError, SystemInput},
};
use gmt_dos_clients_io::{gmt_m2::asm::M2ASMVoiceCoilsMotion, optics::SegmentPiston};
use gmt_dos_clients_lom::LinearOpticalModel;
use gmt_dos_clients_scope::server::{Monitor, Scope, XScope};
use interface::{Data, Read, Update, Write};
use io::{M1SegmentPiston, M2RBSegmentPiston, M2SegmentMeanActuator, M2SegmentPiston};
pub use m1_lom::M1Lom;
pub use m2_lom::M2Lom;
pub use m2rb_lom::M2RBLom;

use crate::{N_ACTUATOR, N_SCOPE};

#[derive(Default, Debug, Clone)]
pub struct M2SegmentActuatorAverage {
    data: Arc<Vec<Arc<Vec<f64>>>>,
}
impl Update for M2SegmentActuatorAverage {}
impl Read<M2ASMVoiceCoilsMotion> for M2SegmentActuatorAverage {
    fn read(&mut self, data: Data<M2ASMVoiceCoilsMotion>) {
        self.data = data.into_arc();
    }
}
impl Write<M2SegmentMeanActuator> for M2SegmentActuatorAverage {
    fn write(&mut self) -> Option<Data<M2SegmentMeanActuator>> {
        Some(
            self.data
                .iter()
                .map(|data| data.iter().sum::<f64>() / N_ACTUATOR as f64)
                .map(|x| x * 1e9)
                .collect::<Vec<f64>>()
                .into(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Scopes {
    m1_lom: Actor<M1Lom, 1, N_SCOPE>,
    m2_lom: Actor<M2Lom, 1, N_SCOPE>,
    m2rb_lom: Actor<M2RBLom, 1, N_SCOPE>,
    lom: Actor<LinearOpticalModel, 1, N_SCOPE>,
    m2_segment_actuator_average: Actor<M2SegmentActuatorAverage, 1, N_SCOPE>,
    segment_piston_scope: Terminator<XScope<SegmentPiston<-9>>, N_SCOPE>,
    m1_segment_piston_scope: Terminator<XScope<M1SegmentPiston>, N_SCOPE>,
    m2_segment_piston_scope: Terminator<XScope<M2SegmentPiston>, N_SCOPE>,
    m2rb_segment_piston_scope: Terminator<XScope<M2RBSegmentPiston>, N_SCOPE>,
    m2_segment_mean_actuator_scope: Terminator<XScope<M2SegmentMeanActuator>, N_SCOPE>,
}

impl Scopes {
    pub fn new(sim_sampling_frequency: f64, monitor: &mut Monitor) -> anyhow::Result<Self> {
        let lom = LinearOpticalModel::new()?;
        let sampling_frequency = sim_sampling_frequency / N_SCOPE as f64;
        Ok(Self {
            m1_lom: M1Lom::from(lom.clone()).into(),
            m2_lom: M2Lom::from(lom.clone()).into(),
            m2rb_lom: M2RBLom::from(lom.clone()).into(),
            lom: lom.into(),
            m2_segment_actuator_average: M2SegmentActuatorAverage::default().into(),
            segment_piston_scope: Scope::<SegmentPiston<-9>>::builder(monitor)
                .sampling_frequency(sampling_frequency)
                .build()?
                .into(),
            m1_segment_piston_scope: Scope::<M1SegmentPiston>::builder(monitor)
                .sampling_frequency(sampling_frequency)
                .build()?
                .into(),
            m2_segment_piston_scope: Scope::<M2SegmentPiston>::builder(monitor)
                .sampling_frequency(sampling_frequency)
                .build()?
                .into(),
            m2rb_segment_piston_scope: Scope::<M2RBSegmentPiston>::builder(monitor)
                .sampling_frequency(sampling_frequency)
                .build()?
                .into(),
            m2_segment_mean_actuator_scope: Scope::<M2SegmentMeanActuator>::builder(monitor)
                .sampling_frequency(sampling_frequency)
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
    fn build(&mut self) -> Result<&mut Self, SystemError> {
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
        self.m2_segment_actuator_average
            .add_output()
            .build::<M2SegmentMeanActuator>()
            .into_input(&mut self.m2_segment_mean_actuator_scope)?;
        Ok(self)
    }

    fn plain(&self) -> gmt_dos_actors::actor::PlainActor {
        PlainActor::new(self.name())
            .inputs(
                PlainActor::from(&self.m1_lom)
                    .inputs()
                    .unwrap()
                    .into_iter()
                    .chain(PlainActor::from(&self.m2_lom).inputs().unwrap().into_iter())
                    .chain(
                        PlainActor::from(&self.m2rb_lom)
                            .inputs()
                            .unwrap()
                            .into_iter(),
                    )
                    .chain(PlainActor::from(&self.lom).inputs().unwrap().into_iter())
                    .chain(
                        PlainActor::from(&self.m2_segment_actuator_average)
                            .inputs()
                            .unwrap()
                            .into_iter(),
                    )
                    .collect::<Vec<_>>(),
            )
            .graph(self.graph())
            .build()
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
            Box::new(&self.m2_segment_actuator_average as &dyn Check),
            Box::new(&self.m1_segment_piston_scope as &dyn Check),
            Box::new(&self.m2_segment_piston_scope as &dyn Check),
            Box::new(&self.m2rb_segment_piston_scope as &dyn Check),
            Box::new(&self.segment_piston_scope as &dyn Check),
            Box::new(&self.m2_segment_mean_actuator_scope as &dyn Check),
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
            Box::new(self.m2_segment_actuator_average) as Box<dyn Task>,
            Box::new(self.m1_segment_piston_scope) as Box<dyn Task>,
            Box::new(self.m2_segment_piston_scope) as Box<dyn Task>,
            Box::new(self.m2rb_segment_piston_scope) as Box<dyn Task>,
            Box::new(self.segment_piston_scope) as Box<dyn Task>,
            Box::new(self.m2_segment_mean_actuator_scope) as Box<dyn Task>,
        ]
        .into_iter()
    }
}

impl SystemInput<M1Lom, 1, N_SCOPE> for Scopes {
    fn input(&mut self) -> &mut Actor<M1Lom, 1, N_SCOPE> {
        &mut self.m1_lom
    }
}

impl SystemInput<M2Lom, 1, N_SCOPE> for Scopes {
    fn input(&mut self) -> &mut Actor<M2Lom, 1, N_SCOPE> {
        &mut self.m2_lom
    }
}

impl SystemInput<M2RBLom, 1, N_SCOPE> for Scopes {
    fn input(&mut self) -> &mut Actor<M2RBLom, 1, N_SCOPE> {
        &mut self.m2rb_lom
    }
}

impl SystemInput<LinearOpticalModel, 1, N_SCOPE> for Scopes {
    fn input(&mut self) -> &mut Actor<LinearOpticalModel, 1, N_SCOPE> {
        &mut self.lom
    }
}

impl SystemInput<M2SegmentActuatorAverage, 1, N_SCOPE> for Scopes {
    fn input(&mut self) -> &mut Actor<M2SegmentActuatorAverage, 1, N_SCOPE> {
        &mut self.m2_segment_actuator_average
    }
}
