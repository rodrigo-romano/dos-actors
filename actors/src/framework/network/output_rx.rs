use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use interface::{Data, UniqueIdentifier, Write};

type Rx<U> = flume::Receiver<Data<U>>;

/// Output signature
///
/// [OutputRx] contains the data of an actor output
/// that is necessary to create the associated input for
/// the receiving actor
pub struct OutputRx<U, C, const NI: usize, const NO: usize>
where
    U: UniqueIdentifier,
    C: Write<U>,
{
    pub actor: String,
    pub output: String,
    pub hash: u64,
    pub rxs: Vec<Rx<U>>,
    pub client: Arc<tokio::sync::Mutex<C>>,
}

impl<U, CO, const NO: usize, const NI: usize> std::error::Error for OutputRx<U, CO, NI, NO>
where
    U: 'static + UniqueIdentifier,
    CO: Write<U>,
{
}
impl<U, CO, const NO: usize, const NI: usize> Display for OutputRx<U, CO, NI, NO>
where
    U: 'static + UniqueIdentifier,
    CO: Write<U>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let OutputRx { actor, output, .. } = self;
        writeln!(
            f,
            r#"TryIntoInputs for output "{}" of actor "{}", check output multiplex #"#,
            output, actor
        )
    }
}
impl<U, CO, const NO: usize, const NI: usize> Debug for OutputRx<U, CO, NI, NO>
where
    U: 'static + UniqueIdentifier,
    CO: Write<U>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(&self, f)
    }
}
