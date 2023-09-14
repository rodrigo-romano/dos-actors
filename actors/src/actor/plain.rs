use crate::{
    io::{InputObject, OutputObject},
    trim,
};

#[derive(Debug, Hash)]
#[doc(hidden)]
pub struct IOData {
    pub name: String,
    pub hash: u64,
}
impl IOData {
    pub fn new(name: String, hash: u64) -> Self {
        Self { name, hash }
    }
}
#[derive(Debug, Hash)]
#[doc(hidden)]
pub enum IO {
    Bootstrap(IOData),
    Regular(IOData),
    Unbounded(IOData),
}
#[derive(Debug, Hash)]
#[doc(hidden)]
pub struct PlainActor {
    pub client: String,
    pub inputs_rate: usize,
    pub outputs_rate: usize,
    pub inputs: Option<Vec<IO>>,
    pub outputs: Option<Vec<IO>>,
    pub hash: u64,
    pub image: Option<String>,
}

impl From<&Box<dyn InputObject>> for IO {
    fn from(value: &Box<dyn InputObject>) -> Self {
        if let Some(_) = value.capacity() {
            IO::Regular(IOData::new(value.who(), value.get_hash()))
        } else {
            IO::Unbounded(IOData::new(value.who(), value.get_hash()))
        }
    }
}

impl From<&Box<dyn OutputObject>> for IO {
    fn from(value: &Box<dyn OutputObject>) -> Self {
        if value.bootstrap() {
            IO::Bootstrap(IOData::new(value.who(), value.get_hash()))
        } else {
            IO::Regular(IOData::new(value.who(), value.get_hash()))
        }
    }
}

impl IO {
    pub fn as_formatted_input(&self, actor_hash: u64, color: usize) -> String {
        match self {
            IO::Bootstrap(input) => format!(
                r#"{0} -> {1} [label="{2}", color={3}, style=bold];"#,
                input.hash,
                actor_hash,
                trim(&input.name),
                color
            ),
            IO::Regular(input) => format!(
                r#"{0} -> {1} [label="{2}", color={3}];"#,
                input.hash,
                actor_hash,
                trim(&input.name),
                color
            ),
            IO::Unbounded(input) => format!(
                r#"{0} -> {1} [label="{2}", color={3}, style=dashed];"#,
                input.hash,
                actor_hash,
                trim(&input.name),
                color
            ),
        }
    }
    pub fn as_formatted_output(&self, actor_hash: u64, color: usize) -> String {
        match self {
            IO::Bootstrap(output) => format!(
                r"{0} -> {1} [color={2}, style=bold];",
                actor_hash, output.hash, color
            ),
            IO::Regular(output) => {
                format!("{0} -> {1} [color={2}];", actor_hash, output.hash, color)
            }
            IO::Unbounded(output) => format!(
                r"{0} -> {1} [color={2}, style=dashed];",
                actor_hash, output.hash, color
            ),
        }
    }
}
