# ASYNC WIND LOADING

Applies all CFD wind loads (in parallel) to the mount and M1 and M2 segments and saves M1 and M2 rigid body motion time series.

The FEM should include the mount FEM that correspond to the final design with the control system to match.

The following environment variables are used:
 * FEM_REPO: sets to the path to the FEM model
 * CFD_REPO: sets to the path to the CFD data

## Model block diagram

  ![](mountloading.png)

## Instructions to run the mountloading example

 1. Merge the latest stable version of the dos-actors crate from the git [repository](https://github.com/rconan/dos-actors).
 1. Save the telescope structural model (zip file) in an appropriate location. After the FDR integration, model 20230530_1756_zen_30_M1_202110_FSM_202305_Mount_202305_noStairs (available from \\drobo-im.gmto.org\im\Christoph) is indicated for wind load end-to-end simulations.
 1. Set the FEM_REPO environmental variable with the path to the structural model.
 1. The git repository of the gmt-fem crate is a submodule inside the dos-actors git repository.        Therefore, if folder dos-actors/clients/fem/model is empty, use `git submodule update –init`. Otherwise, update the folder using `git submodule update`.
 1. Locate the CFD data and set the environmental variable CFD_REPO with the path to the wind load CFD data. Instructions to access the CFD wind loads remote drive are found [here](https://github.com/rconan/grim).
 1. While the 17Hz AZ transfer function issue is under investigation, use the AZ adjusted controller by setting the corresponding instrumental variable, i.e. `export MOUNT_FDR_AZ17HZ=true`
 1. From folder dos-actors/clients/windloads, build the code as: `cargo build --release --example async_mountloading`
 1. Run the code: `cargo run --release --example async_mountloading`
 1. Simulation results are saved as a parquet file: `windloading.parquet`. 