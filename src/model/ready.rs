use super::{Model, Ready, Running};
use chrono::{DateTime, Local, SecondsFormat};
use std::{marker::PhantomData, time::Instant};

impl Model<Ready> {
    /// Spawns each actor task
    pub fn run(mut self) -> Model<Running> {
        let now: DateTime<Local> = Local::now();
        println!(
            "[{}<{}>] LAUNCHED",
            self.name
                .as_ref()
                .unwrap_or(&String::from("Model"))
                .to_uppercase(),
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
        );
        let mut actors = self.actors.take().unwrap();
        let mut task_handles = vec![];
        while let Some(mut actor) = actors.pop() {
            task_handles.push(tokio::spawn(async move {
                actor.task().await;
            }));
        }
        Model::<Running> {
            name: self.name,
            actors: None,
            task_handles: Some(task_handles),
            state: PhantomData,
            start: Instant::now(),
        }
    }
}
