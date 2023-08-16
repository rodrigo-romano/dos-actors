# `gmt_dos-clients_m2-ctrl` tests

## ASMS standalone

```
cargo test --release --features serde --package gmt_dos-clients_m2-ctrl --test asms -- asms --exact --nocapture
```

## ASMS with mount and M1 controller

```
RUST_LOG=info cargo test --release --features serde --package gmt_dos-clients_m2-ctrl --test mount-m1-m2  -- main --exact --nocapture
```