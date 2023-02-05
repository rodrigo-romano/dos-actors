use crate::{Actor, ActorError, Result, Update, Who};
use async_trait::async_trait;
use std::fmt::Display;

use super::PlainActor;

#[async_trait]
pub trait Task: Display + Send {
    /// Runs the [Actor] infinite loop
    ///
    /// The loop ends when the client data is [None] or when either the sending of receiving
    /// end of a channel is dropped
    async fn async_run(&mut self) -> Result<()>;
    /// Run the actor loop in a dedicated thread
    fn spawn(self) -> tokio::task::JoinHandle<()>;
    /**
    Validates the inputs

    Returns en error if there are some inputs but the inputs rate is zero
    or if there are no inputs and the inputs rate is positive
    */
    fn check_inputs(&self) -> Result<()>;
    /**
    Validates the outputs

    Returns en error if there are some outputs but the outputs rate is zero
    or if there are no outputs and the outputs rate is positive
    */
    fn check_outputs(&self) -> Result<()>;
    /// Run the actor loop
    async fn task(&mut self);
    fn n_inputs(&self) -> usize;
    fn n_outputs(&self) -> usize;
    fn inputs_hashes(&self) -> Vec<u64>;
    fn outputs_hashes(&self) -> Vec<u64>;
    fn as_plain(&self) -> PlainActor;
}

#[async_trait]
impl<C, const NI: usize, const NO: usize> Task for Actor<C, NI, NO>
where
    C: 'static + Update + Send,
{
    /// Run the actor loop in a dedicated thread
    fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.task().await;
        })
    }
    /// Run the actor loop
    async fn task(&mut self) {
        match self.bootstrap().await {
            Err(e) => {
                crate::print_info(format!("{} bootstrapping failed", Who::who(self)), Some(&e))
            }
            Ok(_) => {
                crate::print_info(
                    format!("{} loop started", Who::who(self)),
                    None::<&dyn std::error::Error>,
                );
                if let Err(e) = self.async_run().await {
                    crate::print_info(format!("{} loop ended", Who::who(self)), Some(&e));
                }
            }
        }
    }

    /// Starts the actor infinite loop
    async fn async_run(&mut self) -> Result<()> {
        match (self.inputs.as_ref(), self.outputs.as_ref()) {
            (Some(_), Some(_)) => {
                if NO >= NI {
                    // Decimation
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
    fn check_inputs(&self) -> Result<()> {
        match self.inputs {
            Some(_) if NI == 0 => Err(ActorError::SomeInputsZeroRate(Who::who(self))),
            None if NI > 0 => Err(ActorError::NoInputsPositiveRate(Who::who(self))),
            _ => Ok(()),
        }
    }
    fn check_outputs(&self) -> Result<()> {
        match self.outputs {
            Some(_) if NO == 0 => Err(ActorError::SomeOutputsZeroRate(Who::who(self))),
            None if NO > 0 => Err(ActorError::NoOutputsPositiveRate(Who::who(self))),
            _ => Ok(()),
        }
    }
    fn n_inputs(&self) -> usize {
        self.inputs.as_ref().map_or(0, |i| i.len())
    }
    fn n_outputs(&self) -> usize {
        self.outputs
            .as_ref()
            .map_or(0, |o| o.iter().map(|o| o.len()).sum())
    }
    fn inputs_hashes(&self) -> Vec<u64> {
        self.inputs.as_ref().map_or(Vec::new(), |inputs| {
            inputs.iter().map(|input| input.get_hash()).collect()
        })
    }
    fn outputs_hashes(&self) -> Vec<u64> {
        self.outputs.as_ref().map_or(Vec::new(), |outputs| {
            outputs
                .iter()
                .flat_map(|output| vec![output.get_hash(); output.len()])
                .collect()
        })
    }
    fn as_plain(&self) -> PlainActor {
        self.into()
    }
}
