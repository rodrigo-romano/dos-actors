use std::{marker::PhantomData, time::Instant};

use chrono::{DateTime, Local, SecondsFormat};

use crate::actor::{PlainActor, Task, TaskError};

use super::{Model, Running, Unknown};

type Result<T> = std::result::Result<T, TaskError>;

#[async_trait::async_trait]
impl Task for Model<Unknown> {
    async fn async_run(&mut self) -> Result<()> {
        unimplemented!("async_run")
    }

    fn check_inputs(&self) -> Result<()> {
        self.actors
            .iter()
            .flatten()
            .map(|actor| {
                actor.check_inputs()?;
                Ok(())
            })
            .collect()
    }

    fn check_outputs(&self) -> Result<()> {
        self.actors
            .iter()
            .flatten()
            .map(|actor| {
                actor.check_outputs()?;
                Ok(())
            })
            .collect()
    }

    async fn task(&mut self) -> Result<()> {
        let now: DateTime<Local> = Local::now();
        self.verbose.then(|| {
            eprintln!(
                "[{}<{}>] LAUNCHED",
                self.name
                    .as_ref()
                    .unwrap_or(&String::from("Model"))
                    .to_uppercase(),
                now.to_rfc3339_opts(SecondsFormat::Secs, true),
            )
        });
        let task_handles: Vec<_> = self
            .actors
            .take()
            .unwrap()
            .into_iter()
            .map(|mut actor| tokio::spawn(async move { actor.task().await }))
            .collect();
        Model::<Running> {
            name: self.name.clone(),
            actors: None,
            task_handles: Some(task_handles),
            state: PhantomData,
            start: Instant::now(),
            verbose: self.verbose,
        }
        .await?;
        Ok(())
    }

    fn n_inputs(&self) -> usize {
        self.actors
            .iter()
            .flatten()
            .map(|actor| actor.n_inputs())
            .sum()
    }

    fn n_outputs(&self) -> usize {
        self.actors
            .iter()
            .flatten()
            .map(|actor| actor.n_outputs())
            .sum()
    }

    fn inputs_hashes(&self) -> Vec<u64> {
        self.actors
            .iter()
            .flatten()
            .flat_map(|actor| actor.inputs_hashes())
            .collect::<Vec<_>>()
    }

    fn outputs_hashes(&self) -> Vec<u64> {
        self.actors
            .iter()
            .flatten()
            .flat_map(|actor| actor.outputs_hashes())
            .collect::<Vec<_>>()
    }

    fn as_plain(&self) -> crate::actor::PlainActor {
        let mut subsystem = PlainActor::default();
        subsystem.client = self.get_name();
        let iter = self.actors.iter().flatten().map(|actor| actor.as_plain());
        iter.clone()
            .find(|plain| plain.client.contains("Gateway") && plain.client.contains("Ins"))
            .map(|p| {
                subsystem.inputs_rate = p.inputs_rate;
                subsystem.inputs = p.inputs;
            });
        iter.clone()
            .find(|plain| plain.client.contains("Gateway") && plain.client.contains("Outs"))
            .map(|p| {
                subsystem.outputs_rate = p.outputs_rate;
                subsystem.outputs = p.outputs;
            });

        subsystem
    }
}
