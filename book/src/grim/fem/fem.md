# Finite Element Model

The GMT finite element model is loaded from the zip file: `modal_state_space_model_2ndOrder.zip`.
The path to the zip file must be affected to the environment variable: `FEM_REPO`.
The zip file is created with the Matlab script [unwrapFEM](https://github.com/rconan/fem/blob/main/tools/unwrapFEM.m) using data produced by the GMT Integrated Modeling team.

The FEM model is stored into the `gmt-fem` crate as a continuous second order ODE and the `gmt_dos-clients_fem` crate transforms the FEM into discrete 2x2 state space models with as many model as the number of eigen modes of the FEM.

|||||
|-|-|-|-|
|`gmt_dos-clients_fem`| [crates.io](https://crates.io/crates/gmt_dos-clients_fem) | [docs.rs](https://docs.rs/gmt_dos-clients_fem) | [github](https://github.com/rconan/dos-actors/tree/main/clients/fem) |
|`gmt-fem`| [crates.io](https://crates.io/crates/gmt-fem) | [docs.rs](https://docs.rs/gmt-fem) | [github](https://github.com/rconan/fem) |