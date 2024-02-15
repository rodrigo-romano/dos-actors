/*!
# M2 RBMS

In this example, we move the ASMS S1 reference body by 1 micron along the local z-axis (Tz)
and record the rigid body motions and axial displacements of the ASMS S1 facesheet

Run the example with:

```bash
cargo run --release --example m2-rbms --features s8000d002ze30 --no-default-features
```

and post-process with:

```python
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

df = pd.read_parquet("model-data_40.parquet")
plt.figure();plt.plot(np.vstack(df["M2RigidBodyMotions"]));

df = pd.read_parquet("model-data_3999.parquet")
plt.figure();plt.plot(np.vstack(df["FaceSheetFigure#1"])[-1,:],'.');
```
*/

use std::{env, path::Path};

use gmt_dos_actors::actorscript;
use gmt_dos_clients::{select::Select, Sampler, Signals};
use gmt_dos_clients_io::gmt_m2::{asm::segment::FaceSheetFigure, M2RigidBodyMotions};
use gmt_dos_clients_servos::{asms_servo, AsmsServo, GmtM2Hex, GmtServoMechanisms};
use gmt_fem::FEM;

const ACTUATOR_RATE: usize = 80;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    env::set_var(
        "DATA_REPO",
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"),
    );

    let sim_sampling_frequency = 8000;
    // let sim_duration = 1_usize; // second
    let n_step = 4000;

    let fem = FEM::from_env()?;

    // M2 S1 Tz=1e-6m
    let m2_rbm: Signals =
        Signals::new(42, n_step).channel(2, gmt_dos_clients::Signal::Constant(1e-6));

    // GMT Servo-mechanisms system
    let gmt_servos =
        GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(sim_sampling_frequency as f64, fem)
            .asms_servo(AsmsServo::new().facesheet(asms_servo::Facesheet::new()))
            .build()?;

    let rbm_sampler = Sampler::<Vec<f64>, M2RigidBodyMotions>::default();
    let shell_sampler = Sampler::<Vec<f64>, FaceSheetFigure<1>>::default();

    let t_xyz = Select::new(0..3);

    actorscript! {
        1: m2_rbm[M2RigidBodyMotions] -> {gmt_servos::GmtM2Hex}
        1: {gmt_servos::GmtFem}[M2RigidBodyMotions] -> rbm_sampler
        40: rbm_sampler[M2RigidBodyMotions] -> t_xyz[M2RigidBodyMotions]${3}
        1: {gmt_servos::GmtFem}[FaceSheetFigure<1>] -> shell_sampler
        3999: shell_sampler[FaceSheetFigure<1>]${675}
    }

    Ok(())
}
