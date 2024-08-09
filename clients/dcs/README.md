# Integrated Model DCS

A generic implementation of the GMT DCS for the GMT Integrated Model.

## Mount trajectory application 

To run the application, the FEM model must be downloaded first and processed with
```
cargo run --release --bin gmt-fem
```

The app is run with:
```
. setup.sh
cargo run --release --bin mount
```