use crate::{
    io::{self, Assoc},
    Actor, UniqueIdentifier, Update,
};

use super::{ActorOutputBuilder, AddOuput, Rx};

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
    fn build<U>(self) -> (&'a mut Actor<C, NI, NO>, Vec<Rx<U>>)
    where
        C: 'static + Update + Send + io::Write<U>,
        U: 'static + Send + Sync + UniqueIdentifier,
        Assoc<U>: Send + Sync,
    {
        use io::{Output, S};
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
}
