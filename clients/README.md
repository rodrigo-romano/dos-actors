# Actor clients

Repository of clients for GMT integrated model actors.

All clients are using the same interface to read from actor inputs and write to actor outputs.
The client-to-actor interface is defined in the `gmt_dos-clients` crate in the [interface](interface/README.md) directory.

The clients are:

 * [arrow](arrow/README.md): data logger that uses [Apache Arrow](https://arrow.apache.org/) data format and [Parquet](https://parquet.apache.org/) data file
 * [crseo](crseo/README.md): client for the optical model crate [crseo](https://crates.io/crates/crseo)
 * [domeseeing](domeseeing/README.md): client for importing dome seeing wavefront error maps
 * [fem](fem/README.md): client for the GMT FEM crate [gmt-fem](https://crates.io/crates/gmt-fem)
 * [lom](lom/README.md): client for the GMT Linear Optical Model crate [gmt-lom](https://crates.io/crates/gmt-lom)
 * [m1-ctrl](m1-ctrl/README.md): client for the GMT M1 control system
 * [m2-ctrl](m2-ctrl/README.md): client for the GMT M2 control system
 * [mount](mount/README.md): client for the GMT mount control system
 * [scope](scope/README.md): graphical interface for actor output signals
 * [transceiver](transceiver/README.md): client for remote communication between actors
 * [windloads](windloads/README.md): client for importing GMT CFD time series of wind forces and torques

A library of clients inputs/outputs type identifiers is implemented in the `gmt_dos-clients_io` crate in the [io](io/README.md) directory.
