use std::{marker::PhantomData, time::Instant};

use async_trait::async_trait;
use chrono::{DateTime, Local, SecondsFormat};

use crate::{
    actor::PlainActor,
    framework::model::{Check, Task, TaskError},
    model::{Model, Running, Unknown},
};

use super::System;

type Result<T> = std::result::Result<T, TaskError>;

#[async_trait]
impl<T> Task for T
where
    T: System + Send + Sync + Check,
{
    async fn async_run(&mut self) -> Result<()> {
        todo!()
    }

    async fn task(mut self: Box<Self>) -> Result<()> {
        /* match *self {
            Self {
                name,
                system,
                gateway: mut way_inandout,
                ..
            } => {
                let now: DateTime<Local> = Local::now();
                eprintln!(
                    "[{}<{}>] LAUNCHED",
                    name.as_ref()
                        .unwrap_or(&String::from("SubSystem"))
                        .to_uppercase(),
                    now.to_rfc3339_opts(SecondsFormat::Secs, true),
                );
                let h_in = tokio::spawn(async move {
                    {
                        /*                         if let (Some(outputs), Some(inputs)) =
                            (&mut way_in.outputs, &mut way_in.inputs)
                        {
                            Self::bootstrap_gateways(outputs, inputs).await?;
                        } */
                        way_inandout.async_run().await
                    }
                });
                /*                 let h_out = tokio::spawn(async move {
                    {
                        /*                         if let (Some(outputs), Some(inputs)) =
                            (&mut way_out.outputs, &mut way_out.inputs)
                        {
                            Self::bootstrap_gateways(outputs, inputs).await?;
                        } */
                        way_out.async_run().await
                    }
                }); */
                let mut model: Model<Unknown> = self.system().into();
                let mut task_handles: Vec<_> = model
                    .actors
                    .take()
                    .unwrap()
                    .into_iter()
                    .map(|actor| tokio::spawn(async move { actor.task().await }))
                    .collect();
                task_handles.append(&mut vec![h_in]);
                Model::<Running> {
                    name,
                    actors: None,
                    task_handles: Some(task_handles),
                    state: PhantomData,
                    start: Instant::now(),
                    verbose: true,
                    elapsed_time: Default::default(),
                }
            }
        }
        .await?; */

        let name = self.name();
        let now: DateTime<Local> = Local::now();
        eprintln!(
            "[{}<{}>] LAUNCHED",
            name.as_ref()
                .unwrap_or(&String::from("SubSystem"))
                .to_uppercase(),
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
        );
        let mut model: Model<Unknown> = (*self).into();
        let task_handles: Vec<_> = model
            .actors
            .take()
            .unwrap()
            .into_iter()
            .map(|actor| tokio::spawn(async move { actor.task().await }))
            .collect();
        Model::<Running> {
            name,
            actors: None,
            task_handles: Some(task_handles),
            state: PhantomData,
            start: Instant::now(),
            verbose: true,
            elapsed_time: Default::default(),
        }
        .await?;
        Ok(())
    }

    fn as_plain(&self) -> PlainActor {
        <Self as Check>::_as_plain(&self)
    }
}
