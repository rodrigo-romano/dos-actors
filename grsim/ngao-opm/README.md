# NGAO OPM

Integrated model for the GMT Natural Guide Star Observatory Performance Mode including
 * the mount control system (8kHz),
 * the M1 hardpoints to actuators force loop control system (80Hz),
 * the ASMS inner loop control system (8kHz),
 * the pyramid (1kHz) and the HDFS (10Hz) optical models and control algorithm,
 * atmospheric turbulence.

The FEM model is `20230131_1605_zen_30_M1_202110_ASM_202208_Mount_20211` discretized at 8kHz.

The model is run with:
```shell
export FEM_REPO=<path-to-fem>/20230131_1605_zen_30_M1_202110_ASM_202208_Mount_20211
ACTORS_GRAPH=twopi RUST_LOG=info cargo run --release
```

or as root:
```shell
sudo -E RUST_LOG=info LD_LIBRARY_PATH=/usr/local/cuda-10.0/lib64/  ./../../../target/release/ngao-opm
```

## Environment variables

| Var | Comment | Defaut |
|-----|---------|-------:|
| FEM_REPO | path to the the FEM data folder | - |
| CFD_REPO | path to the CFD cases folder | - |
| GMT_MODES_PATH | path to the ceo files folder | - |
| DATA_REPO | path to the results folder | - |
| N_KL_MODE | # of Karhunen-Loeve | 66 |
| ZA | zenith angle [deg] | 30 |
| AZ | azimuth angle [deg] | 0 |
| VS | vent and wind screen config | os |
| WS | wind speed [m/s] | 7 |
| SIM_DURATION | simulation duration [s] | 10HDFS |
| HSV | Hankel singular values threshold | - |