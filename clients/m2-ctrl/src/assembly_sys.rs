use std::fmt::Display;

use gmt_dos_actors::{
    actor::{Actor, PlainActor},
    framework::model::Check,
    system::{System, SystemInput, SystemOutput},
    Task,
};
use gmt_dos_clients_io::Assembly;

mod dispatch;
mod inner_controllers;
pub use dispatch::{DispatchIn, DispatchOut};
pub use inner_controllers::AsmsInnerControllers;

impl<const R: usize> Assembly for ASMS<R> {}

#[derive(Debug, Clone)]
pub struct ASMS<const R: usize = 1>
where
    Self: Assembly,
{
    segments: Vec<AsmsInnerControllers<R>>,
    dispatch_in: Actor<DispatchIn, R, R>,
    dispatch_out: Actor<DispatchOut, R, R>,
}

impl<'a, const R: usize> IntoIterator for &'a ASMS<R> {
    type Item = Box<&'a dyn Check>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments
            .iter()
            .map(|x| x.as_check())
            .chain(
                vec![
                    Box::new(&self.dispatch_in as &dyn Check),
                    Box::new(&self.dispatch_out as &dyn Check),
                ]
                .into_iter(),
            )
            .collect::<Vec<_>>()
            .into_iter()
    }
}
impl<const R: usize> IntoIterator for Box<ASMS<R>> {
    type Item = Box<dyn Task>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.segments
            .into_iter()
            .map(|x| x.into_task())
            .chain(
                vec![
                    Box::new(self.dispatch_in) as Box<dyn Task>,
                    Box::new(self.dispatch_out) as Box<dyn Task>,
                ]
                .into_iter(),
            )
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<const R: usize> ASMS<R> {
    pub fn new(n_mode: Vec<usize>, ks: Vec<Option<Vec<f64>>>) -> Self {
        Self {
            segments: n_mode
                .into_iter()
                .zip(ks.into_iter())
                .zip(<ASMS<R> as Assembly>::SIDS.into_iter())
                .map(|((n_mode, ks), sid)| AsmsInnerControllers::new(sid, n_mode, ks))
                .collect::<Vec<_>>(),
            dispatch_in: DispatchIn::new().into(),
            dispatch_out: DispatchOut::new().into(),
        }
    }
}

/* impl<const R: usize> GetField for ASMS<R> {
    fn get_field(&self, idx: usize) -> Option<&dyn Check> {
        match idx {
            n if n < <ASMS<R> as Assembly>::N => {
                self.segments.get(idx).and_then(|segment| segment.get())
            }
            n if n == <ASMS<R> as Assembly>::N => Some(&self.dispatch_in as &dyn Check),
            n if n == <ASMS<R> as Assembly>::N + 1 => Some(&self.dispatch_out as &dyn Check),
            _ => None,
        }
    }
}

impl<const R: usize> From<ASMS<R>> for Model<Unknown> {
    fn from(asms: ASMS<R>) -> Self {
        asms.segments
            .into_iter()
            .fold(Model::default(), |model, segment| {
                model + segment.into_model()
            })
            + asms.dispatch_in
            + asms.dispatch_out
    }
}

impl<const R: usize> gateway::Gateways for ASMS<R> {
    type DataType = Vec<Arc<Vec<f64>>>;

    const N_IN: usize = 2;

    const N_OUT: usize = 2;
} */

impl<const R: usize> Display for ASMS<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<const R: usize> System for ASMS<R> {
    fn build(&mut self) -> anyhow::Result<&mut Self> {
        // gateway_in
        //     .add_output()
        //     .build::<M2ASMAsmCommand>()
        //     .into_input(&mut self.dispatch_in)?;
        // gateway_in
        //     .add_output()
        //     .build::<M2ASMVoiceCoilsMotion>()
        //     .into_input(&mut self.dispatch_in)?;
        self.segments
            .iter_mut()
            .map(|segment| segment.asm_command(&mut self.dispatch_in))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.asm_voice_coils_motion(&mut self.dispatch_in))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.asm_voice_coils_forces(&mut self.dispatch_out))
            .collect::<anyhow::Result<Vec<()>>>()?;
        self.segments
            .iter_mut()
            .map(|segment| segment.asm_fluid_damping_forces(&mut self.dispatch_out))
            .collect::<anyhow::Result<Vec<()>>>()?;
        // self.dispatch_out
        //     .add_output()
        //     .build::<M2ASMVoiceCoilsForces>()
        //     .into_input(gateway_out)?;
        // self.dispatch_out
        //     .add_output()
        //     .build::<M2ASMFluidDampingForces>()
        //     .into_input(gateway_out)?;
        Ok(self)
    }

    fn plain(&self) -> gmt_dos_actors::actor::PlainActor {
        let mut plain = PlainActor::default();
        plain.client = self.name();
        plain.inputs_rate = 1;
        plain.outputs_rate = 1;
        plain.inputs = PlainActor::from(&self.dispatch_in).inputs;
        plain.outputs = PlainActor::from(&self.dispatch_out).outputs;
        plain
    }

    fn name(&self) -> String {
        format!("ASMS<{R}>")
    }
}

impl<const R: usize> SystemInput<DispatchIn, R, R> for ASMS<R> {
    fn input(&mut self) -> &mut Actor<DispatchIn, R, R> {
        &mut self.dispatch_in
    }
}

impl<const R: usize> SystemOutput<DispatchOut, R, R> for ASMS<R> {
    fn output(&mut self) -> &mut Actor<DispatchOut, R, R> {
        &mut self.dispatch_out
    }
}
