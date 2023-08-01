# GMT DOS Actors Transceiver Example

The transmitter is run with:
```rust
RUST_LOG=info cargo run --example tx
```
and the receiver with 
```rust
RUST_LOG=info cargo run --example rx
```
The receiver should print the following sequence `[0,1,0,-1,0,1,0]` for `sin` and `[0,-1,0,1,0,-1,0]` for `isin`.