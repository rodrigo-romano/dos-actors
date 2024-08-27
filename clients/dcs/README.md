# Integrated Model DCS

A generic implementation of the GMT DCS for the GMT Integrated Model.

## Mount trajectory application 

To run the application, the FEM model must be [downloaded](https://gmtocorp-my.sharepoint.com/:u:/g/personal/rconan_gmto_org/EThI5QQjnjJMtdxmyTcxgpYBZ_yo1hO_VvGm5Xs178vOkQ?e=mFdkZY) first and processed with
```
. setup.sh
cargo run --release --bin gmt-fem
```

The app is run with:
```
. setup.sh
cargo run --release --bin im-dcs-mount
```