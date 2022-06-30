#[derive(Debug, Hash)]
#[doc(hidden)]
pub struct PlainIO {
    pub name: String,
    pub hash: u64,
}
impl PlainIO {
    pub fn new(name: String, hash: u64) -> Self {
        Self { name, hash }
    }
}
#[derive(Debug, Hash)]
#[doc(hidden)]
pub enum PlainOutput {
    Bootstrap(PlainIO),
    Regular(PlainIO),
}
#[derive(Debug, Hash)]
#[doc(hidden)]
pub struct PlainActor {
    pub client: String,
    pub inputs_rate: usize,
    pub outputs_rate: usize,
    pub inputs: Option<Vec<PlainIO>>,
    pub outputs: Option<Vec<PlainOutput>>,
    pub hash: u64,
}
