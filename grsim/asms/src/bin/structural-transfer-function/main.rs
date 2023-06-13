use std::fs::File;

use asms::{Frequencies, FrequencyResponse, Structural};
use clap::Parser;

/// Evaluate the average transfer function between some inputs and some outputs of a structural model.
/// The location of the structural model is given by the FEM_REPO environment variable
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Comma separated list of FEM inputs
    #[arg(short, long, value_delimiter = ',')]
    inputs: Vec<String>,
    /// Comma separated list of FEM outputs
    #[arg(short, long, value_delimiter = ',')]
    outputs: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let model = Structural::builder(args.inputs, args.outputs).build()?;

    let nu = Frequencies::logspace(1f64, 500f64, 1000);
    let (nu, resp) = model.frequency_response(nu);

    let mean_resp: Vec<f64> = resp
        .into_iter()
        .map(|resp| resp.column_mean().row_mean().as_slice()[0].norm())
        .collect();

    let mut file = File::create("struct-tf.pkl")?;
    serde_pickle::to_writer(&mut file, &(nu, mean_resp), Default::default())?;

    Ok(())
}
