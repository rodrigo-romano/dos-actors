use super::{Actors, Model, ModelError, Ready, Result, Unknown};
use std::{marker::PhantomData, ops::Add, time::Instant};

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
