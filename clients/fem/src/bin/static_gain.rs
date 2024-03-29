//! FEM STATIC GAIN
//!
//! Computes the FEM static gain between given inputs and outputs.
//!
//! M2 S7 gain matrix for the voice coils:
//! ```shell
//! cargo run -r -p gmt_dos-clients_fem --bin static_gain --features="serde clap" -- \
//!     -i MC_M2_S7_VC_delta_F -o MC_M2_S7_VC_delta_D \
//!     -f m2s7_vc_gain.pkl
//! ```
//!
//! M2 S1 & S7 gain matrix for the voice coils:
//! ```shell
//! cargo run -r -p gmt_dos-clients_fem --bin static_gain --features="serde clap" -- \
//!     -i MC_M2_S1_VC_delta_F -i MC_M2_S7_VC_delta_F \
//!     -o MC_M2_S1_VC_delta_D -o MC_M2_S7_VC_delta_D \
//!     -f m2s1-7_vc_gain.pkl
//! ```

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
