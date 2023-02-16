use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    path::Path,
};

use crate::actor::PlainActor;

/// [Model](crate::model::Model) network mapping
///
/// The structure is used to build a [Graphviz](https://www.graphviz.org/) diagram of a [Model](crate::model::Model).
/// A new [Graph] is created with `Model::graph()`.
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
                            output.as_formatted_output(actor.hash, *color)
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
                            input.as_formatted_input(actor.hash, *color)
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
                .map(|actor| if let Some(image) = actor.image.as_ref() {
                    format!(
                        r#"{} [label="{}", labelloc=t, image="{}"]"#,
                        actor.hash, actor.client, image
                    )
                } else {
                    format!(r#"{} [label="{}"]"#, actor.hash, actor.client)
                })
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
