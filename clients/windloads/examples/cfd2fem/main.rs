/*!
# CFD2FEM

Save mount, M1 and M2 FEM windloads inputs into a parquet file

Running the example from `dos-actors/clients/windloads`
```shell
MOUNT_MODEL=MOUNT_FDR_1kHz \
FEM_REPO=~/mnt/20250506_1715_zen_30_M1_202110_FSM_202305_Mount_202305_pier_202411_M1_actDamping/ \
DATA_REPO=`pwd`/examples/cfd2fem \
cargo r -r --example cfd2fem
```

*/

use gmt_dos_actors::actorscript;
use gmt_dos_clients_io::{cfd_wind_loads::*, gmt_fem::inputs::CFD2025046F};
use gmt_dos_clients_windloads::CfdLoads;
use gmt_fem::FEM;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // loading the FEM at FEM_REPO
    let mut fem = FEM::from_env()?;
    println!("{fem}");
    // loading the CFD monitors from the current directory,
    // resampling the time series at 1kHz for 1s
    // and using the default mount, M1 and M2 CFD windload ouptputs.
    // Note that the size of the FEM CFD input has been reduced slightly
    let cfd_loads = CfdLoads::foh(".", 1000)
        .duration(1.0)
        .windloads(&mut fem, Default::default())
        .build()?;

    // printing the descriptions of the FEM CFD input
    // restricted to Fx only
    let loads_index =
        <gmt_fem::FEM as gmt_dos_clients_fem::Model>::in_position::<CFD2025046F>(&fem).unwrap();
    let descriptions: Vec<_> = fem.inputs[loads_index]
        .as_ref()
        .map(|i| i.get_by(|x| Some(x.descriptions.clone())))
        .unwrap()
        .into_iter()
        .step_by(6)
        .collect();
    println!(
        "FEM CFD2025046F input [{}]:\n{:}",
        6 * descriptions.len(),
        descriptions.join("\n")
    );

    // playing the FEM windloads and saving them into `model_data_1.parquet`
    actorscript!(
      1: cfd_loads[CFDMountWindLoads]$
      1: cfd_loads[CFDM1WindLoads]$
      1: cfd_loads[CFDM2WindLoads]$
    );

    Ok(())
}
