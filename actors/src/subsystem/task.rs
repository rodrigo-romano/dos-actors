use std::{marker::PhantomData, time::Instant};

use async_trait::async_trait;

use crate::{
    actor::PlainActor,
    model::{Model, Running, Unknown},
    Check, Task, TaskError,
};

use super::{BuildSystem, Gateways, SubSystem, SubSystemIterator};

type Result<T> = std::result::Result<T, TaskError>;

#[async_trait]
impl<M, const NI: usize, const NO: usize> Task for SubSystem<M, NI, NO>
where
    M: Gateways + BuildSystem<M, NI, NO> + 'static,
    Model<Unknown>: From<M>,
    for<'a> SubSystemIterator<'a, M>: Iterator<Item = &'a dyn Check>,
{
    async fn async_run(&mut self) -> Result<()> {
        todo!()
    }

    async fn task(mut self: Box<Self>) -> Result<()> {
        match *self {
            Self {
                name,
                system,
                gateway_in: mut way_in,
                gateway_out: mut way_out,
            } => {
                let h_in = tokio::spawn(async move {
                    {
/*                         if let (Some(outputs), Some(inputs)) =
                            (&mut way_in.outputs, &mut way_in.inputs)
                        {
                            Self::bootstrap_gateways(outputs, inputs).await?;
                        } */
                        way_in.async_run().await
                    }
                });
                let h_out = tokio::spawn(async move {
                    {
/*                         if let (Some(outputs), Some(inputs)) =
                            (&mut way_out.outputs, &mut way_out.inputs)
                        {
                            Self::bootstrap_gateways(outputs, inputs).await?;
                        } */
                        way_out.async_run().await
                    }
                });
                let mut model: Model<Unknown> = system.into();
                let mut task_handles: Vec<_> = model
                    .actors
                    .take()
                    .unwrap()
                    .into_iter()
                    .map(|actor| tokio::spawn(async move { actor.task().await }))
                    .collect();
                task_handles.append(&mut vec![h_in, h_out]);
                Model::<Running> {
                    name,
                    actors: None,
                    task_handles: Some(task_handles),
                    state: PhantomData,
                    start: Instant::now(),
                    verbose: true,
                }
            }
        }
        .await?;

        Ok(())
    }

    fn as_plain(&self) -> PlainActor {
        <Self as Check>::_as_plain(&self)
    }
}