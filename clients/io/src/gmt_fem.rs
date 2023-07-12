//! GMT Finite Element Model

/// FEM inputs
pub mod inputs {
    include!(concat!(env!("OUT_DIR"), "/fem_actors_inputs.rs"));
}
/// FEM outputs
pub mod outputs {
    include!(concat!(env!("OUT_DIR"), "/fem_actors_outputs.rs"));
}
