use super::{Completed, Model, ModelError, Result, Running};
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
            task_handle.await?;
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
