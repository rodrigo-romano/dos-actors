use std::fs::File;

use clap::Parser;
use gmt_dos_clients_fem::{Model, Switch};
use gmt_fem::FEM;

#[derive(Parser, Debug)]
pub struct Cli {
    /// FEM inputs
    #[arg(short, long)]
    inputs: Vec<String>,
    /// FEM outputs
    #[arg(short, long)]
    outputs: Vec<String>,
    /// static gain filename
    #[arg(short, long)]
    filename: String,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut fem = FEM::from_env()?;

    fem.switch_inputs(Switch::Off, None)
        .switch_outputs(Switch::Off, None);
    let k = fem
        .switch_inputs_by_name(args.inputs, Switch::On)
        .and_then(|fem| fem.switch_outputs_by_name(args.outputs, Switch::On))
        .map(|fem| fem.reduced_static_gain().unwrap())?;
    serde_pickle::to_writer(&mut File::create(args.filename)?, &k, Default::default())?;

    Ok(())
}
