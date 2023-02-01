use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    path::Path,
};

use crate::actor::{plain::PlainOutput, PlainActor};

/// [Model] network mapping
///
/// The structure is used to build a [Graphviz](https://www.graphviz.org/) diagram of a [Model].
/// A new [Graph] is created with [Model::graph()].
///
/// The model flow chart is written to a SVG image with `neato -Gstart=rand -Tsvg filename.dot > filename.svg`
#[derive(Debug)]
pub struct Graph {
    actors: Vec<PlainActor>,
}
impl Graph {
    pub(super) fn new(actors: Vec<PlainActor>) -> Self {
        let mut hasher = DefaultHasher::new();
        let mut actors = actors;
        actors.iter_mut().for_each(|actor| {
            actor.client = actor
                .client
                .replace("::Controller", "")
                .split('<')
                .next()
                .unwrap()
                .split("::")
                .last()
                .unwrap()
                .to_string();
            actor.hash(&mut hasher);
            actor.hash = hasher.finish();
        });
        Self { actors }
    }
    /// Returns the diagram in the [Graphviz](https://www.graphviz.org/) dot language
    pub fn to_string(&self) -> String {
        use PlainOutput::*;
        let mut lookup: HashMap<usize, usize> = HashMap::new();
        let mut colors = (1usize..=8).cycle();
        let outputs: Vec<_> = self
            .actors
            .iter()
            .filter_map(|actor| {
                actor.outputs.as_ref().map(|outputs| {
                    outputs
                        .iter()
                        .map(|output| {
                            let color = lookup
                                .entry(actor.outputs_rate)
                                .or_insert_with(|| colors.next().unwrap());
                            match output {
                                Bootstrap(output) => format!(
                                    r"{0} -> {1} [color={2}, style=bold];",
                                    actor.hash, output.hash, color
                                ),
                                Regular(output) => format!(
                                    "{0} -> {1} [color={2}];",
                                    actor.hash, output.hash, color
                                ),
                            }
                        })
                        .collect::<Vec<String>>()
                })
            })
            .flatten()
            .collect();
        let inputs: Vec<_> = self
            .actors
            .iter()
            .filter_map(|actor| {
                actor.inputs.as_ref().map(|inputs| {
                    inputs
                        .iter()
                        .map(|input| {
                            let color = lookup
                                .entry(actor.inputs_rate)
                                .or_insert_with(|| colors.next().unwrap());
                            format!(
                                r#"{0} -> {1} [label="{2}", color={3}];"#,
                                input.hash,
                                actor.hash,
                                input.name.split("::").last().unwrap(),
                                color
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
  bgcolor = gray24;
  {{node [shape=box, width=1.5, style="rounded,filled", fillcolor=lightgray]; {};}}
  node [shape=point, fillcolor=gray24, color=lightgray];

  /* Outputs */
{{
  edge [arrowhead=none,colorscheme=dark28];
  {}
}}
  /* Inputs */
{{
  edge [arrowhead=vee,fontsize=9, fontcolor=lightgray, labelfloat=true,colorscheme=dark28]
  {}
}}
}}
"#,
            self.actors
                .iter()
                .map(|actor| format!(r#"{} [label="{}"]"#, actor.hash, actor.client))
                .collect::<Vec<String>>()
                .join("; "),
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
