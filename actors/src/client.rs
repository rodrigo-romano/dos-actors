use std::{marker::PhantomData, sync::Arc};

use interface::Update;
use tokio::sync::Mutex;

use crate::{actor::Actor, ArcMutex};

pub struct Client<'a, T: ArcMutex> {
    client: Arc<Mutex<T>>,
    label: Option<String>,
    image: Option<String>,
    lifetime: PhantomData<&'a T>,
}

impl<'a, T: Update> Client<'a, T> {
    pub fn set_label(&mut self, label: impl ToString) {
        self.label = Some(label.to_string());
    }
    pub fn set_image(&mut self, image: impl ToString) {
        self.image = Some(image.to_string());
    }
}

impl<'a, T: Update> From<T> for Client<'a, T> {
    fn from(value: T) -> Self {
        Self {
            client: value.into_arcx(),
            label: None,
            image: None,
            lifetime: PhantomData,
        }
    }
}

impl<'a, C: Update, const NI: usize, const NO: usize> From<&Client<'a, C>> for Actor<C, NI, NO> {
    fn from(client: &Client<C>) -> Self {
        let actor = Actor::new(client.client.clone());
        match (client.label.as_ref(), client.image.as_ref()) {
            (Some(label), Some(image)) => actor.name(label).image(image),
            (Some(label), None) => actor.name(label),
            (None, Some(image)) => actor.image(image),
            (None, None) => actor,
        }
    }
}

impl<'a, T: ArcMutex> Client<'a, T> {
    pub async fn lock(&'a self) -> tokio::sync::MutexGuard<'a, T> {
        self.client.lock().await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn client() {
        use crate::{
            actor::Actor,
            client::{Client, Update},
        };

        struct TestClient;

        impl Update for TestClient {}

        let test_client = TestClient;

        let client = Client::from(test_client);
        let actor: Actor<_> = Actor::from(&client);

        let other_client: Client<'_, TestClient> = Client::from(client);
    }
}
