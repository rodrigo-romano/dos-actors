# Examples

All examples are using data from the same server, that is run with
```text
RUST_LOG=info cargo run --no-default-features --features server --example tx
```

## async

Asynchronous scope using [tokio](https://tokio.rs/) runtime:

```text
RUST_LOG=info cargo run --example async
```

## async-macro

Same as `async` but defining the scope with a procedural function macro:

```text
RUST_LOG=info cargo run --example async-macro
```