# `gmt_dos-clients_mount`

[![Crates.io](https://img.shields.io/crates/v/gmt_dos-clients_mount.svg)](https://crates.io/crates/gmt_dos-clients_mount)
[![Documentation](https://docs.rs/gmt_dos-clients_mount/badge.svg)](https://docs.rs/gmt_dos-clients_mount/)

A client for the GMT mount control system.

There are a few mount controller and driver models to choose from.
The model selection is set with the `MOUNT_MODEL` environment variable.
Possible values for `MOUNT_MODEL` are:
 * `MOUNT_FDR_1kHz`: FDR version of the controller and driver sampled at 1kHz
 * `MOUNT_PDR_8kHz`: PDR version of the controller and driver sampled at 8kHz
 * `MOUNT_FDR_1kHz-az17Hz`: PDR version of the controller and driver sampled at 8kHz with a notch filter at 17Hz in the controller