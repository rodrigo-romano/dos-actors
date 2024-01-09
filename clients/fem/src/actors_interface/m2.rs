//! M2 CONTROL

#[cfg(fem_with_asm)]
pub mod asm;
#[cfg(fem_with_fsm)]
pub mod fsm;
pub mod positionners;
pub mod rigid_body_motions;
#[doc(hidden)]
pub use super::prelude;
