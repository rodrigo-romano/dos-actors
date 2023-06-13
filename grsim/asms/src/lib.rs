mod frequency_response;
pub use frequency_response::{
    if64, BesselFilter, FirstOrderLowPass, Frequencies, FrequencyResponse, PICompensator,
};
mod structural;
pub use structural::{Structural, StructuralBuilder, StructuralError};
mod asm;
pub use asm::ASM;
mod response;
pub use response::{Sys, MIMO};

pub trait BuilderTrait {
    /// Sets the FEM modal damping coefficient
    fn damping(self, z: f64) -> Self;
    /// Sets the filename where [Structural] is seralize to
    fn filename<S: Into<String>>(self, file_name: S) -> Self;
    /// Enables the compensation of the static gain mismatch
    ///
    /// An optional delay [s] may be added
    fn enable_static_gain_mismatch_compensation(self, maybe_delay: Option<f64>) -> Self;
}
