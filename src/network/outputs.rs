use crate::{
    io::{Output, OutputObject, S},
    Actor, Who,
};
use interface as io;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use interface::{Assoc, UniqueIdentifier, Update};

use super::{ActorOutputBuilder, AddOuput, OutputRx, Rx};

impl Default for ActorOutputBuilder {
    fn default() -> Self {
        Self {
            capacity: Vec::new(),
            bootstrap: false,
        }
    }
}
impl ActorOutputBuilder {
    /// Creates a new actor output builder multiplexed `n` times
    pub fn new(n: usize) -> Self {
        Self {
            capacity: vec![1; n],
            ..Default::default()
        }
    }
}

impl<'a, C, const NI: usize, const NO: usize> AddOuput<'a, C, NI, NO>
    for (&'a mut Actor<C, NI, NO>, ActorOutputBuilder)
where
    C: 'static + Update + Send,
{
    fn unbounded(self) -> Self {
        let n = self.1.capacity.len();
        (
            self.0,
            ActorOutputBuilder {
                capacity: vec![usize::MAX; n],
                ..self.1
            },
        )
    }
    fn bootstrap(self) -> Self {
        (
            self.0,
            ActorOutputBuilder {
                bootstrap: true,
                ..self.1
            },
        )
    }
    fn multiplex(self, n: usize) -> Self {
        (
            self.0,
            ActorOutputBuilder {
                capacity: vec![self.1.capacity[0]; n],
                ..self.1
            },
        )
    }
    fn legacy_build<U>(self) -> (&'a mut Actor<C, NI, NO>, Vec<Rx<U>>)
    where
        C: 'static + Update + Send + io::Write<U>,
        U: 'static + Send + Sync + UniqueIdentifier,
        Assoc<U>: Send + Sync,
    {
        let (actor, builder) = self;
        let mut txs = vec![];
        let mut rxs = vec![];
        for &cap in &builder.capacity {
            let (tx, rx) = if cap == usize::MAX {
                flume::unbounded::<S<U>>()
            } else {
                flume::bounded::<S<U>>(cap)
            };
            txs.push(tx);
            rxs.push(rx);
        }

        let output: Output<C, Assoc<U>, U, NO> = Output::builder(actor.client.clone())
            .bootstrap(builder.bootstrap)
            .senders(txs)
            .build();

        if let Some(ref mut outputs) = actor.outputs {
            outputs.push(Box::new(output));
        } else {
            actor.outputs = Some(vec![Box::new(output)]);
        }

        (actor, rxs)
    }
    fn build<U>(self) -> std::result::Result<(), OutputRx<U, C, NI, NO>>
    where
        C: 'static + Update + Send + io::Write<U>,
        U: 'static + Send + Sync + UniqueIdentifier,
        Assoc<U>: Send + Sync,
    {
        let (actor, builder) = self;
        let mut txs = vec![];
        let mut rxs = vec![];
        for &cap in &builder.capacity {
            let (tx, rx) = if cap == usize::MAX {
                flume::unbounded::<S<U>>()
            } else {
                flume::bounded::<S<U>>(cap)
            };
            txs.push(tx);
            rxs.push(rx);
        }

        let mut output: Output<C, Assoc<U>, U, NO> = Output::builder(actor.client.clone())
            .bootstrap(builder.bootstrap)
            .senders(txs)
            .build();

        let mut hasher = DefaultHasher::new();
        actor.who().hash(&mut hasher);
        let output_name = Who::who(&output);
        output_name
            .split("::")
            .last()
            .unwrap()
            .to_owned()
            .hash(&mut hasher);
        let hash = hasher.finish();
        <Output<C, Assoc<U>, U, NO> as OutputObject>::set_hash(&mut output, hash);

        if let Some(ref mut outputs) = actor.outputs {
            outputs.push(Box::new(output));
        } else {
            actor.outputs = Some(vec![Box::new(output)]);
        }

        Err(OutputRx {
            hash,
            rxs,
            client: std::sync::Arc::clone(&actor.client),
            actor: actor.who(),
            output: output_name,
        })
    }
}
