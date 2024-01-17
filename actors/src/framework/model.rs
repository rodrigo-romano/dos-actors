//! # Model framework
//!
//! The model module define the interface to build a [Model].
//!
//! Any structure that implements [Task], and its super trait [Check], can be part of an actors [Model].
//!
//! [Model]: crate::model::Model

use std::{env, path::Path, process::Command};

use crate::{actor::PlainActor, graph::Graph, model, system::System, ActorError};

#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("error in Task from Actor")]
    FromActor(#[from] ActorError),
    #[error("error in Task from Model")]
    FromModel(#[from] model::ModelError),
}

/// Interface for model verification routines
///
pub trait Check {
    /// Validates the inputs
    ///
    /// Returns en error if there are some inputs but the inputs rate is zero
    /// or if there are no inputs and the inputs rate is positive
    fn check_inputs(&self) -> std::result::Result<(), CheckError>;
    /// Validates the outputs
    ///
    /// Returns en error if there are some outputs but the outputs rate is zero
    /// or if there are no outputs and the outputs rate is positive
    fn check_outputs(&self) -> std::result::Result<(), CheckError>;
    /// Return the number of inputs
    fn n_inputs(&self) -> usize;
    /// Return the number of outputs
    fn n_outputs(&self) -> usize;
    /// Return the hash # of inputs
    fn inputs_hashes(&self) -> Vec<u64>;
    /// Return the hash # of outputs
    fn outputs_hashes(&self) -> Vec<u64>;
    fn _as_plain(&self) -> PlainActor;
    fn is_system(&self) -> bool {
        false
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("error in Task from Actor")]
    FromActor(#[from] ActorError),
    #[error("error in Task from Model")]
    FromModel(#[from] model::ModelError),
}

/// Interface for running model components
#[async_trait::async_trait]
pub trait Task: Check + std::fmt::Display + Send + Sync {
    /// Runs the [Actor](crate::actor::Actor) infinite loop
    ///
    /// The loop ends when the client data is [None] or when either the sending of receiving
    /// end of a channel is dropped
    async fn async_run(&mut self) -> std::result::Result<(), TaskError>;
    /// Run the actor loop in a dedicated thread
    fn spawn(self) -> tokio::task::JoinHandle<std::result::Result<(), TaskError>>
    where
        Self: Sized + 'static,
    {
        tokio::spawn(async move { Box::new(self).task().await })
    }
    /// Run the actor loop
    async fn task(self: Box<Self>) -> std::result::Result<(), TaskError>;
    fn as_plain(&self) -> PlainActor;
}

/// Flowchart name
pub trait GetName {
    /// Returns the flowchart name
    fn get_name(&self) -> String {
        "integrated_model".into()
    }
}

/// Actors flowchart interface
pub trait FlowChart: GetName {
    /// Returns the actors network graph
    fn graph(&self) -> Option<Graph>;
    /// Writes the actors flowchart
    ///
    /// The flowchart file is written either in the current directory
    /// or in the directory give by the environment variable `DATA_REPO`.
    /// The flowchart is created with [Graphviz](https://www.graphviz.org/) neato filter,
    /// other filters can be specified with the environment variable `FLOWCHART`
    fn flowchart(self) -> Self;
}
impl<T: GetName> FlowChart for T
where
    for<'a> &'a T: IntoIterator<Item = PlainActor>,
{
    fn graph(&self) -> Option<Graph> {
        let actors: Vec<_> = self.into_iter().collect();
        if actors.is_empty() {
            None
        } else {
            Some(Graph::new(actors))
        }
    }

    fn flowchart(self) -> Self {
        let name = self.get_name();
        let root_env = env::var("DATA_REPO").unwrap_or_else(|_| ".".to_string());
        let path = Path::new(&root_env).join(&name);
        if let Some(graph) = self.graph() {
            match graph.to_dot(path.with_extension("dot")) {
                Ok(_) => {
                    if let Err(e) =
                        Command::new(env::var("FLOWCHART").unwrap_or("neato".to_string()))
                            .arg("-Gstart=rand")
                            .arg("-Tsvg")
                            .arg("-O")
                            .arg(path.with_extension("dot").to_str().unwrap())
                            .output()
                    {
                        println!(
                            "Failed to convert Graphviz dot file {path:?} to SVG image with {e}"
                        )
                    }
                }
                Err(e) => println!("Failed to write Graphviz dot file {path:?} with {e}"),
            }
        }
        self
    }
}

pub trait SystemFlowChart {
    fn graph(&self) -> Option<Graph>;
    fn flowchart(&self) -> &Self;
}
impl<T: System> SystemFlowChart for T
where
    for<'a> &'a T: IntoIterator<Item = Box<&'a dyn Check>>,
{
    fn graph(&self) -> Option<Graph> {
        let actors: Vec<_> = self.into_iter().map(|x| x._as_plain()).collect();
        if actors.is_empty() {
            None
        } else {
            Some(Graph::new(actors))
        }
    }

    fn flowchart(&self) -> &Self {
        let name = self.get_name();
        let root_env = env::var("DATA_REPO").unwrap_or_else(|_| ".".to_string());
        let path = Path::new(&root_env).join(&name);
        if let Some(graph) = self.graph() {
            match graph.to_dot(path.with_extension("dot")) {
                Ok(_) => {
                    if let Err(e) =
                        Command::new(env::var("FLOWCHART").unwrap_or("neato".to_string()))
                            .arg("-Gstart=rand")
                            .arg("-Tsvg")
                            .arg("-O")
                            .arg(path.with_extension("dot").to_str().unwrap())
                            .output()
                    {
                        println!(
                            "Failed to convert Graphviz dot file {path:?} to SVG image with {e}"
                        )
                    }
                }
                Err(e) => println!("Failed to write Graphviz dot file {path:?} with {e}"),
            }
        }
        &self
    }
}
