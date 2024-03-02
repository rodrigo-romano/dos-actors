use interface::Update;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn serialize<S, C: Update + Serialize>(client: &Arc<Mutex<C>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime for serializing `Actor`");
    let client: &C = &rt.block_on(client.lock());
    client.serialize(s)
}

pub fn deserialize<'de, D, C: Update + Deserialize<'de>>(
    deserializer: D,
) -> Result<Arc<Mutex<C>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = C::deserialize(deserializer)?;
    Ok(Arc::new(Mutex::new(value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Actor;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Client();
    impl Update for Client {}

    #[test]
    fn serde() {
        let actor: Actor<_> = Client().into();
        dbg!(&actor);
        let value = serde_pickle::to_value(&actor).unwrap();
        let actor: Actor<Client, 1, 1> = serde_pickle::from_value(value).unwrap();
        dbg!(&actor);
    }
}
