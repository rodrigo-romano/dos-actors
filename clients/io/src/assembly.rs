//! # GMT segments selection
//!
//! The GMT segments are selected by setting the `ASSEMBLY`
//! environment variable.
//! For example, setting `ASSEMBLY=1,2,7` will select segments #1, 2 and 7.
//! If `ASSEMBLY` is not set, all segments are selected.

include!(concat!(env!("OUT_DIR"), "/assembly.rs"));
