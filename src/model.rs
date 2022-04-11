/*!
# Integrated model

The module implements the high-level integrated model interface.
The model is build from a collection of [actor]s.

The model has 4 states:
 1. [Unknown]: model state at its creation
 2. [Ready]: model state after succesfully performing runtime checks on inputs and outputs on all the actors, the model can move to the [Ready] state only from the [Unknown] state
 3. [Running]: model state while all the actors are performing their respective tasks, the model can move to the [Running] state only from the [Ready] state
 4. [Completed]: model state after the succesful completion of the tasks of all the actors, the model can move to the [Completed] state only from the [Running] state

# Example

A 3 actors model with [Signals], [Sampler] and [Logging] clients is build with:
```
use dos_actors::prelude::*;
let mut source: Initiator<_> = Signals::new(1, 100).into();
enum Source {};
let mut sampler: Actor<_, 1, 10> = Sampler::<Vec<f64>, Source>::default().into();
let logging = Logging::<f64>::default().into_arcx();
let mut sink = Terminator::<_, 10>::new(logging);
```
`sampler` decimates `source` with a 1:10 ratio.
The `source` connects to the `sampler` using the empty enum type `Source` as the data identifier.
The source data is then logged into the client of the `sink` actor.
```
# use dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging);
source.add_output().build::<Vec<f64>, Source>().into_input(&mut sampler);
sampler.add_output().build::<Vec<f64>,Source>().into_input(&mut sink);
```
A [model](crate::model) is build from the set of actors:
```
# use dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Vec<f64>, Source>().into_input(&mut sampler);
# sampler.add_output().build::<Vec<f64>,Source>().into_input(&mut sink);
Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)]);
```
Actors are checked for inputs/outputs consistencies:
```
# use dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Vec<f64>, Source>().into_input(&mut sampler);
# sampler.add_output().build::<Vec<f64>,Source>().into_input(&mut sink);
Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)])
       .check()?;
# Ok::<(), dos_actors::model::ModelError>(())
```
The model run the actor tasks:
```
# tokio_test::block_on(async {
# use dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Vec<f64>, Source>().into_input(&mut sampler);
# sampler.add_output().build::<Vec<f64>,Source>().into_input(&mut sink);
Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)])
       .check()?
       .run();
# Ok::<(), dos_actors::model::ModelError>(())
# });
```
and wait for the tasks to finish:
```
# tokio_test::block_on(async {
# use dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Vec<f64>, Source>().into_input(&mut sampler);
# sampler.add_output().build::<Vec<f64>,Source>().into_input(&mut sink);
Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)])
       .check()?
       .run()
       .wait()
       .await?;
# Ok::<(), dos_actors::model::ModelError>(())
# });
```
Once the model run to completion, the data from `logging` is read with:
```
# tokio_test::block_on(async {
# use dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Vec<f64>, Source>().into_input(&mut sampler);
# sampler.add_output().build::<Vec<f64>,Source>().into_input(&mut sink);
# Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)])
#       .check()?
#       .run()
#       .wait()
#       .await?;
let data: &[f64]  = &logging.lock().await;
# Ok::<(), dos_actors::model::ModelError>(())
# });
```

[actor]: crate::actor
[client]: crate::clients
[Mutex]: tokio::sync::Mutex
[Arc]: std::sync::Arc
[Arcmutex]: crate::ArcMutex
[into_arcx]: crate::ArcMutex::into_arcx
[Signals]: crate::clients::Signals
[Sampler]: crate::clients::Sampler
[Logging]: crate::clients::Logging
*/

use crate::Task;
use std::{fs::File, io::Write, marker::PhantomData, path::Path};

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("no actors found in the model")]
    NoActors,
    #[error("failed to join the task")]
    TaskError(#[from] tokio::task::JoinError),
    #[error("Actor IO inconsistency")]
    ActorIO(#[from] crate::ActorError),
}

type Result<T> = std::result::Result<T, ModelError>;

pub enum Unknown {}
pub enum Ready {}
pub enum Running {}
pub enum Completed {}

type Actors = Vec<Box<dyn Task>>;

/// Actor model
pub struct Model<State> {
    actors: Option<Actors>,
    task_handles: Option<Vec<tokio::task::JoinHandle<()>>>,
    state: PhantomData<State>,
}

impl Model<Unknown> {
    /// Returns a new model
    pub fn new(actors: Actors) -> Self {
        Self {
            actors: Some(actors),
            task_handles: None,
            state: PhantomData,
        }
    }
    /// Validates actors inputs and outputs
    pub fn check(self) -> Result<Model<Ready>> {
        match self.actors {
            Some(ref actors) => {
                for actor in actors {
                    actor.check_inputs()?;
                    actor.check_outputs()?;
                }
                Ok(Model::<Ready> {
                    actors: self.actors,
                    task_handles: None,
                    state: PhantomData,
                })
            }
            None => Err(ModelError::NoActors),
        }
    }
    /// Returns a [Graph] of the model
    pub fn graph(&self) -> Option<Graph> {
        self.actors.as_ref().map(|actors| {
            let clients: Vec<_> = actors.iter().map(|a| a.client_typename()).collect();
            let outputs: Vec<_> = actors.iter().map(|a| a.outputs_typename()).collect();
            let inputs: Vec<_> = actors.iter().map(|a| a.inputs_typename()).collect();
            Graph::new(clients, inputs, outputs)
        })
    }
}

impl Model<Ready> {
    /// Spawns each actor task
    pub fn run(mut self) -> Model<Running> {
        let mut actors = self.actors.take().unwrap();
        let mut task_handles = vec![];
        while let Some(mut actor) = actors.pop() {
            task_handles.push(tokio::spawn(async move {
                actor.task().await;
            }));
        }
        Model::<Running> {
            actors: None,
            task_handles: Some(task_handles),
            state: PhantomData,
        }
    }
}

impl Model<Running> {
    /// Waits for the task of each actor to finish
    pub async fn wait(mut self) -> Result<Model<Completed>> {
        let task_handles = self.task_handles.take().unwrap();
        for task_handle in task_handles.into_iter() {
            task_handle.await?;
        }
        Ok(Model::<Completed> {
            actors: None,
            task_handles: None,
            state: PhantomData,
        })
    }
}

/// [Model] network mapping
///
/// The structure is used to build a [Graphviz](https://www.graphviz.org/) diagram of a [Model].
/// A new [Graph] is created with [Model::graph()].
///
/// The model flow chart is written to a SVG image with `neato -Gstart=rand -Tsvg filename.dot > filename.svg`
pub struct Graph {
    clients: Vec<String>,
    inputs: Vec<Option<Vec<String>>>,
    outputs: Vec<Option<Vec<String>>>,
}
impl Graph {
    fn new(
        clients: Vec<String>,
        inputs: Vec<Option<Vec<String>>>,
        outputs: Vec<Option<Vec<String>>>,
    ) -> Self {
        Self {
            clients,
            inputs,
            outputs,
        }
    }
    /// Returns the diagram in the [Graphviz](https://www.graphviz.org/) dot language
    pub fn to_string(&self) -> String {
        let clients: Vec<_> = self
            .clients
            .iter()
            .map(|c| c.split("::").last().unwrap())
            .collect();
        let outputs: Vec<_> = self
            .outputs
            .iter()
            .zip(&clients)
            .filter_map(|(outputs, client)| {
                outputs.as_ref().map(|outputs| {
                    outputs
                        .iter()
                        .map(|output| {
                            format!("{} -> {};", client, output.split("::").last().unwrap())
                        })
                        .collect::<Vec<String>>()
                })
            })
            .flatten()
            .collect();
        let inputs: Vec<_> = self
            .inputs
            .iter()
            .zip(&clients)
            .filter_map(|(inputs, client)| {
                inputs.as_ref().map(|inputs| {
                    inputs
                        .iter()
                        .map(|input| {
                            format!(
                                r#"{0} -> {1} [label="{0}"];"#,
                                input.split("::").last().unwrap(),
                                client
                            )
                        })
                        .collect::<Vec<String>>()
                })
            })
            .flatten()
            .collect();
        format!(
            r#"
digraph  G {{
  overlap = scale;
  splines = true;
  node [shape=box]; {};
  node [shape=point];

  /* Outputs */
{{
  edge [arrowhead=none];
  {}
}}
{{
  /* Inputs */
  edge [fontsize=9,labelfloat=true]
  {}
}}
}}
"#,
            clients.join("; "),
            outputs.join("\n"),
            inputs.join("\n"),
        )
    }
    /// Writes the output of [Graph::to_string()] to a file
    pub fn to_dot<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;
        write!(&mut file, "{}", self.to_string())?;
        Ok(())
    }
}
