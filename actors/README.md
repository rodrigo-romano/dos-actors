# `gmt_dos-actors`

[![Crates.io](https://img.shields.io/crates/v/gmt_dos-actors.svg)](https://crates.io/crates/gmt_dos-actors)
[![Documentation](https://docs.rs/gmt_dos-actors/badge.svg)](https://docs.rs/gmt_dos-actors/)

gmt_dos-actors is an implementation of the actor model applied to integrated modeling for the Giant Magellan Telescope.

## Features

 * Asynchronous actors model
 * [Channel](https://crates.io/crates/flume) based data exchange between actors
 * channels validation at compile time
 * formal interface definition (trait based) between actors and actor clients 
 * [scripting](dsl/README.md) [macro](https://docs.rs/gmt_dos-actors/latest/gmt_dos_actors/macro.actorscript.html) to reduce boilerplate clutter
