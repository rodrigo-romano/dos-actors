/// Actor outputs builder
#[derive(Default)]
pub struct ActorOutputBuilder {
    capacity: Vec<usize>,
    bootstrap: bool,
}

impl ActorOutputBuilder {
    /// Creates a new actor output builder multiplexed `n` times
    pub fn new(n: usize) -> Self {
        Self {
            capacity: vec![1; n],
            ..Default::default()
        }
    }
    /// Returns the output channel capacity
    pub fn capacity(&self) -> &[usize] {
        self.capacity.as_slice()
    }
    /// Returns the bootstrapping flag
    pub fn is_bootstrap(&self) -> bool {
        self.bootstrap
    }
    /// Sets the output channel capacity to a very large size
    pub fn unbounded(&mut self) {
        self.capacity = vec![usize::MAX; self.capacity.len()];
    }
    /// Sets the bootstrapping flag
    pub fn bootstrap(&mut self) {
        self.bootstrap = true;
    }
    /// Multiplex the output
    pub fn multiplex(&mut self, n: usize) {
        self.capacity = vec![self.capacity[0]; n];
    }
}
