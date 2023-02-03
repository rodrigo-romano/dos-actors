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
use gmt_dos_actors::prelude::*;
let mut source: Initiator<_> = Signals::new(1, 100).into();
#[derive(UID)]
enum Source {};
let mut sampler: Actor<_, 1, 10> = Sampler::<Vec<f64>, Source>::default().into();
let logging = Logging::<f64>::default().into_arcx();
let mut sink = Terminator::<_, 10>::new(logging);
```
`sampler` decimates `source` with a 1:10 ratio.
The `source` connects to the `sampler` using the empty enum type `Source` as the data identifier.
The source data is then logged into the client of the `sink` actor.
```
# use gmt_dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# #[derive(UID)]
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging);
source.add_output().build::<Source>().into_input(&mut sampler);
sampler.add_output().build::<Source>().into_input(&mut sink);
```
A [model](crate::model) is build from the set of actors:
```
# use gmt_dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# #[derive(UID)]
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Source>().into_input(&mut sampler);
# sampler.add_output().build::<Source>().into_input(&mut sink);
Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)]);
```
Actors are checked for inputs/outputs consistencies:
```
# use gmt_dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# #[derive(UID)]
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Source>().into_input(&mut sampler);
# sampler.add_output().build::<Source>().into_input(&mut sink);
Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)])
       .check()?;
# Ok::<(), gmt_dos_actors::model::ModelError>(())
```
The model run the actor tasks:
```
# tokio_test::block_on(async {
# use gmt_dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# #[derive(UID)]
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Source>().into_input(&mut sampler);
# sampler.add_output().build::<Source>().into_input(&mut sink);
Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)])
       .check()?
       .run();
# Ok::<(), gmt_dos_actors::model::ModelError>(())
# });
```
and wait for the tasks to finish:
```
# tokio_test::block_on(async {
# use gmt_dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# #[derive(UID)]
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Source>().into_input(&mut sampler);
# sampler.add_output().build::<Source>().into_input(&mut sink);
Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)])
       .check()?
       .run()
       .wait()
       .await?;
# Ok::<(), gmt_dos_actors::model::ModelError>(())
# });
```
Once the model run to completion, the data from `logging` is read with:
```
# tokio_test::block_on(async {
# use gmt_dos_actors::prelude::*;
# let mut source: Initiator<_> = Signals::new(1, 100).into();
# #[derive(UID)]
# enum Source {};
# let mut sampler: Actor<_> = Sampler::<Vec<f64>, Source>::default().into();
# let logging = Logging::<f64>::default().into_arcx();
# let mut sink = Terminator::<_>::new(logging.clone());
# source.add_output().build::<Source>().into_input(&mut sampler);
# sampler.add_output().build::<Source>().into_input(&mut sink);
# Model::new(vec![Box::new(source), Box::new(sampler), Box::new(sink)])
#       .check()?
#       .run()
#       .wait()
#       .await?;
let data: &[f64]  = &logging.lock().await;
# Ok::<(), gmt_dos_actors::model::ModelError>(())
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
use chrono::{DateTime, Local, SecondsFormat};
use std::{
    env, fmt::Display, marker::PhantomData, ops::Add, path::Path, process::Command, time::Instant,
};

mod flowchart;
pub use flowchart::Graph;

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

/// [Model] initial state
pub enum Unknown {}
/// Valid [Model] state
pub enum Ready {}
/// [Model]ing in-progress state
pub enum Running {}
/// [Model] final state
pub enum Completed {}

type Actors = Vec<Box<dyn Task>>;

/// Actor model
pub struct Model<State> {
    name: Option<String>,
    actors: Option<Actors>,
    task_handles: Option<Vec<tokio::task::JoinHandle<()>>>,
    state: PhantomData<State>,
    start: Instant,
}

impl<S> Display for Model<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} [{},{:?}]",
            self.name.as_ref().unwrap_or(&"ACTOR MODEL".to_string()),
            self.n_actors(),
            self.n_io()
        )?;
        if let Some(actors) = &self.actors {
            for actor in actors {
                write!(f, " {}", actor)?;
            }
        }
        Ok(())
    }
}

impl<S> Model<S> {
    /// Prints some informations about the model and the actors within
    pub fn inspect(self) -> Self {
        println!("{self}");
        self
    }
    /// Returns the total number of inputs and the total number of outputs
    ///
    /// Both numbers should be the same
    pub fn n_io(&self) -> (usize, usize) {
        if let Some(ref actors) = self.actors {
            actors
                .iter()
                .fold((0usize, 0usize), |(mut i, mut o), actor| {
                    i += actor.n_inputs();
                    o += actor.n_outputs();
                    (i, o)
                })
        } else {
            (0, 0)
        }
    }
    /// Returns the number of actors
    pub fn n_actors(&self) -> usize {
        self.actors.as_ref().map_or(0, |actors| actors.len())
    }
}

#[doc(hidden)]
pub trait UnknownOrReady {}
impl UnknownOrReady for Unknown {}
impl UnknownOrReady for Ready {}
impl<State> Model<State>
where
    State: UnknownOrReady,
{
    /// Returns a [Graph] of the model
    pub fn graph(&self) -> Option<Graph> {
        self.actors
            .as_ref()
            .map(|actors| Graph::new(actors.iter().map(|a| a.as_plain()).collect()))
    }
    /// Produces the model flowchart from [Graph]
    ///
    /// The flowchart is written to the SVG file "integrated_model.dot.svg".
    /// If a different model `name` is set, the file gets written to "`name`.dot.svg"
    pub fn flowchart(self) -> Self {
        let name = self
            .name
            .clone()
            .unwrap_or_else(|| "integrated_model".to_string());
        let root_env = env::var("DATA_REPO").unwrap_or_else(|_| ".".to_string());
        let path = Path::new(&root_env).join(&name);
        if let Some(graph) = self.graph() {
            match graph.to_dot(path.with_extension("dot")) {
                Ok(_) => {
                    if let Err(e) = Command::new("neato")
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

impl Model<Unknown> {
    /// Returns a new model
    pub fn new(actors: Actors) -> Self {
        Self {
            name: None,
            actors: Some(actors),
            task_handles: None,
            state: PhantomData,
            start: Instant::now(),
        }
    }
    /// Sets the model name
    pub fn name<S: Into<String>>(self, name: S) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }
    /// Validates actors inputs and outputs
    pub fn check(self) -> Result<Model<Ready>> {
        let (n_inputs, n_outputs) = self.n_io();
        assert_eq!(
            n_inputs, n_outputs,
            "I/O #({},{}) don't match, did you forget to add some actors to the model?",
            n_inputs, n_outputs
        );
        match self.actors {
            Some(ref actors) => {
                let mut inputs_hashes = vec![];
                let mut outputs_hashes = vec![];
                for actor in actors {
                    actor.check_inputs()?;
                    actor.check_outputs()?;
                    inputs_hashes.append(&mut actor.inputs_hashes());
                    outputs_hashes.append(&mut actor.outputs_hashes());
                }
                let hashes_diff = outputs_hashes
                    .into_iter()
                    .zip(inputs_hashes.into_iter())
                    .map(|(o, i)| o as i128 - i as i128)
                    .sum::<i128>();
                assert_eq!(hashes_diff,0i128,
                "I/O hashes difference: expected 0, found {}, did you forget to add some actors to the model?",
                hashes_diff);
                Ok(Model::<Ready> {
                    name: self.name,
                    actors: self.actors,
                    task_handles: None,
                    state: PhantomData,
                    start: Instant::now(),
                })
            }
            None => Err(ModelError::NoActors),
        }
    }
}

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

impl Model<Running> {
    /// Waits for the task of each actor to finish
    pub async fn wait(mut self) -> Result<Model<Completed>> {
        let task_handles = self.task_handles.take().unwrap();
        for task_handle in task_handles.into_iter() {
            task_handle.await?;
        }
        let elapsed_time = Instant::now().duration_since(self.start);
        let now: DateTime<Local> = Local::now();
        println!(
            "[{}<{}>] COMPLETED in {}",
            self.name
                .as_ref()
                .unwrap_or(&String::from("Model"))
                .to_uppercase(),
            now.to_rfc3339_opts(SecondsFormat::Secs, true),
            humantime::format_duration(elapsed_time)
        );
        Ok(Model::<Completed> {
            name: self.name,
            actors: None,
            task_handles: None,
            state: PhantomData,
            start: Instant::now(),
        })
    }
}

use std::future::{Future, IntoFuture};
use std::pin::Pin;
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

impl Add for Model<Unknown> {
    type Output = Model<Unknown>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.actors, rhs.actors) {
            (None, None) => Model::new(vec![]),
            (None, Some(b)) => Model::new(b),
            (Some(a), None) => Model::new(a),
            (Some(mut a), Some(mut b)) => {
                a.append(&mut b);
                Model::new(a)
            }
        }
    }
}
