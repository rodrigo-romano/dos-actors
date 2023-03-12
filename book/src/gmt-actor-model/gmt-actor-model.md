# GMT Actors Model

A GMT integrated model is a collection of actors, each actor executing a specific task or set of tasks and exchanging data at predefined sampling rates.

The GMT Actors Model is distributed among 2 crates: `gmt_dos-actors` and `gmt_dos-clients`.

`gmt_dos-actors` implements the actor model including the methods to send and receive data to and from actors and the higher level abstraction of a model.

The interface between a client and the inputs and outputs of an actor is defined in the `gmt_dos-clients` crate.
The crate also provides a set of clients with the `gmt_dos-actors` interface already setup.

To use `gmt_dos-actors`, add it to your list of dependencies with 
```
cargo add gmt_dos-actors
```
and import the contents of the prelude module:
```rust,,no_run,noplayground
use gmt_dos_actors::prelude::*;
```

To use some of the clients in `gmt_dos-clients`, add the crate to your list of dependencies with 
```
cargo add gmt_dos-clients
```
If you are only looking for the `gmt_dos-actors` interface, you can instead do
```
cargo add gmt_dos-clients --no-default-features --features interface
```

|||||
|||||
|`gmt_dos-actors`| [crates.io](https://crates.io/crates/gmt_dos-actors) | [docs.rs](https://docs.rs/gmt_dos-actors/6.0.0) | [github](https://github.com/rconan/dos-actors) |
|`gmt_dos-clients`| [crates.io](https://crates.io/crates/gmt_dos-clients) | [docs.rs](https://docs.rs/gmt_dos-clients/1.0.0) | [github](https://github.com/rconan/dos-actors/tree/main/clients) |