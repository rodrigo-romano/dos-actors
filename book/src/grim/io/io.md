# IO

The `IO` crate provides the types for the inputs and outputs of the clients associated with the GMT integrated model.

DOS Client

|||||
|-|-|-|-|
|`gmt_dos-clients_io`| [crates.io](https://crates.io/crates/gmt_dos-clients_io) | [docs.rs](https://docs.rs/gmt_dos-clients_io) | [github](https://github.com/rconan/dos-actors/tree/main/clients/io) |

Remark: The definition of the inputs and outputs of the FEM actor has moved to the `gmt_dos-clients_io` crate, since version 2.4.0. So, to see the list of *inputs* and *outputs* of a particular telescope structural model, one should set `FEM_REPO` to the proper location and, from the `dos-actors` repository folder, run
> `cargo doc --package gmt_dos-clients_io --no-deps --open`
Note that the doc will also display the descriptions as written by @Christoph in *inputTable* and *outputTable*.
