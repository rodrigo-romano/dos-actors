use std::any::type_name;

use async_trait::async_trait;
use interface::Update;

use crate::framework::model::{Task, TaskError};

use super::{Actor, PlainActor};

type Result<T> = std::result::Result<T, TaskError>;

#[async_trait]
impl<C, const NI: usize, const NO: usize> Task for Actor<C, NI, NO>
where
    C: 'static + Update,
{
    /// Run the actor loop
    async fn task(mut self: Box<Self>) -> Result<()> {
        /*         match self.bootstrap().await {
            Err(e) => crate::print_info(
                format!("{} bootstrapping failed", Who::highlight(self)),
                Some(&e),
            ),
            Ok(_) => {
                crate::print_info(
                    format!("{} loop started", Who::highlight(self)),
                    None::<&dyn std::error::Error>,
                );
                if let Err(e) = self.async_run().await {
                    println!(
                        "{}{:?}",
                        format!("{} loop ended", Who::highlight(self)),
                        Some(&e)
                    );
                }
            }
        } */
        self.async_run().await
    }

    /// Starts the actor infinite loop
    async fn async_run(&mut self) -> Result<()> {
        log::debug!("ACTOR LOOP ({NI}/{NO}): {}", type_name::<C>());
        let _bootstrap = self.bootstrap().await?;
        match (self.inputs.as_ref(), self.outputs.as_ref()) {
            (Some(_), Some(_)) => {
                if NO >= NI {
                    // Decimation
                    // if !bootstrap {
                    self.collect().await?.client.lock().await.update();
                    self.distribute().await?;
                    // } else {
                    //     log::debug!("BOOTSTRAPPING ACTOR LOOP ({NI}/{NO}): {}", type_name::<C>());
                    //     self.collect().await?.client.lock().await.update();
                    //     self.distribute().await?;
                    // }
                    loop {
                        for _ in 0..NO / NI {
                            self.collect().await?.client.lock().await.update();
                        }
                        self.distribute().await?;
                    }
                } else {
                    // Upsampling
                    loop {
                        self.collect().await?.client.lock().await.update();
                        for _ in 0..NI / NO {
                            self.distribute().await?;
                        }
                    }
                }
            }
            (None, Some(_)) => loop {
                // Initiator
                self.client.lock().await.update();
                self.distribute().await?;
            },
            (Some(_), None) => loop {
                // Terminator
                self.collect().await?.client.lock().await.update();
            },
            (None, None) => Ok(()),
        }
    }

    fn as_plain(&self) -> PlainActor {
        self.into()
    }
}
