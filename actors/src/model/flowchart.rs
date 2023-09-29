use crate::{actor::PlainActor, framework::model::GetName};

use super::{Model, UnknownOrReady};

impl<State> IntoIterator for &Model<State>
where
    State: UnknownOrReady,
{
    type Item = PlainActor;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.actors
            .as_ref()
            .map(|actors| {
                actors
                    .iter()
                    .map(|a| a.as_plain())
                    .collect::<Vec<PlainActor>>()
            })
            .unwrap_or_default()
            .into_iter()
    }
}

impl<State> GetName for Model<State>
where
    State: UnknownOrReady,
{
    fn get_name(&self) -> String {
        self.name
            .as_ref()
            .map_or("integrated_model", |x| x.as_str())
            .into()
    }
}
