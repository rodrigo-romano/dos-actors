use std::any::type_name;

use interface::Update;

use crate::{
    actor::PlainActor,
    framework::model::{Check, GetName},
    subsystem::{GetField, Iter, SubSystemIterator},
};

use super::{Gateways, System};

impl<T> IntoIterator for &T
where
    T: System + Gateways + GetField,
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
        self.iter()
            .map(|actors| actors._as_plain())
            .collect::<Vec<PlainActor>>()
            .into_iter()
    }
}

impl<T> GetName for T
where
    T: System,
{
    fn get_name(&self) -> String {
        self.name()
            .as_ref()
            .map_or(type_name::<T>(), |x| x.as_str())
            .into()
    }
}
