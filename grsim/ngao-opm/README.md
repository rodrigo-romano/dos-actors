# NGAO OPM

Integrated model for the GMT Natural Guide Star Observatory Performance Mode including
 * the mount control system (8kHz),
 * the M1 hardpoints to actuators force loop control system (80Hz),
 * the ASMS inner loop control system (8kHz),
 * the pyramid (1kHz) and the HDFS (10Hz) optical models and control algorithm,
 * atmospheric turbulence.

The FEM model is `20230131_1605_zen_30_M1_202110_ASM_202208_Mount_20211` discretized at 8kHz.

The model is run with:
```
export FEM_REPO=<path-to-fem>/20230131_1605_zen_30_M1_202110_ASM_202208_Mount_20211
ACTORS_GRAPH=twopi RUST_LOG=info cargo run --release
```
