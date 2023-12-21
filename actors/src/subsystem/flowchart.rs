use std::any::type_name;

use crate::{
    actor::PlainActor,
    framework::model::{Check, GetName},
};

use super::{
    subsystem::{Built, State},
    Gateways, Iter, SubSystem, SubSystemIterator,
};

impl<M, const NI: usize, const NO: usize> IntoIterator for &SubSystem<M, NI, NO, Built>
where
    M: Gateways + Clone + 'static,
    for<'a> SubSystemIterator<'a, M>: Iterator<Item = &'a dyn Check>,
{
    type Item = PlainActor;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        /*        std::iter::once(self.gateway_in._as_plain())
        .chain(
            self.system
                .iter()
                .map(|actors| actors._as_plain())
                .collect::<Vec<PlainActor>>()
                .into_iter(),
        )
        .chain(std::iter::once(self.gateway_out._as_plain()))
        .collect::<Vec<PlainActor>>()
        .into_iter() */
        self.system
            .iter()
            .map(|actors| actors._as_plain())
            .collect::<Vec<PlainActor>>()
            .into_iter()
    }
}

impl<M, S, const NI: usize, const NO: usize> GetName for SubSystem<M, NI, NO, S>
where
    M: Gateways + Clone,
    S: State,
{
    fn get_name(&self) -> String {
        self.name
            .as_ref()
            .map_or(type_name::<M>(), |x| x.as_str())
            .into()
    }
}
