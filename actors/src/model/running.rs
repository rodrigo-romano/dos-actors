use super::{Completed, Model, ModelError, Result, Running};
use crate::{
    framework::model::TaskError::FromActor,
    ActorError::{Disconnected, DropRecv, DropSend},
};
use chrono::{DateTime, Local, SecondsFormat};
use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
    time::Instant,
};

impl Model<Running> {
    /// Waits for the task of each actor to finish
    pub async fn wait(mut self) -> Result<Model<Completed>> {
        let task_handles = self.task_handles.take().unwrap();
        for task_handle in task_handles.into_iter() {
            // task_handle.await?.map_err(|e| Box::new(e))?;
            match task_handle.await? {
                Ok(_) => {
                    log::info!(
                        "{} succesfully completed",
                        self.name.as_ref().unwrap_or(&String::from("Model"))
                    );
                    Ok(())
                }
                Err(FromActor(Disconnected(msg))) => {
                    log::info!("{} has been disconnected", msg);
                    Ok(())
                }
                Err(FromActor(DropRecv { msg, .. })) => {
                    log::info!("{} has been dropped", msg);
                    Ok(())
                }
                Err(FromActor(DropSend { msg, .. })) => {
                    log::info!("{} has been dropped", msg);
                    Ok(())
                }
                Err(e) => Err(e),
            }
            .map_err(|e| Box::new(e))?;
        }
        let elapsed_time = Instant::now().duration_since(self.start);
        let now: DateTime<Local> = Local::now();
        self.verbose.then(|| {
            eprintln!(
                "[{}<{}>] COMPLETED in {}",
                self.name
                    .as_ref()
                    .unwrap_or(&String::from("Model"))
                    .to_uppercase(),
                now.to_rfc3339_opts(SecondsFormat::Secs, true),
                humantime::format_duration(elapsed_time)
            )
        });
        Ok(Model::<Completed> {
            name: self.name,
            actors: None,
            task_handles: None,
            state: PhantomData,
            start: Instant::now(),
            verbose: self.verbose,
        })
    }
}

pub type ModelCompleted = Pin<
    Box<dyn Future<Output = std::result::Result<Model<Completed>, ModelError>> + Send + 'static>,
>;
impl IntoFuture for Model<Running> {
    type IntoFuture = ModelCompleted;
    type Output = <ModelCompleted as Future>::Output;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.wait())
    }
}
