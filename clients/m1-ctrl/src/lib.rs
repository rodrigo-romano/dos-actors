//! # M1 control system

mod actuators;
pub use actuators::Actuators;
pub use hardpoints::{Hardpoints, LoadCells};

#[cfg(fem)]
mod calibration;
mod hardpoints;
#[cfg(fem)]
mod segment_builder;
#[cfg(fem)]
pub use calibration::Calibration;
// mod builder;
// use builder::Builder;

pub struct Segment<const ID: u8, const ACTUATOR_RATE: usize>;
impl<const ID: u8, const ACTUATOR_RATE: usize> Segment<ID, ACTUATOR_RATE> {
    pub fn new() -> Self {
        Self
    }
}

pub struct Mirror<const ACTUATOR_RATE: usize> {}
