mod segment;
use std::ops::{Deref, DerefMut};

use gmt_dos_actors::subsystem::SubSystem;
pub use segment::SegmentControl;

use crate::Calibration;

pub struct Segment<const S: u8, const R: usize>(SubSystem<SegmentControl<S, R>>);

impl<const S: u8, const R: usize> Segment<S, R> {
    pub fn new(calibration: &Calibration) -> anyhow::Result<Self> {
        Ok(Segment(
            SubSystem::new(SegmentControl::new(calibration)).build()?,
        ))
    }
}

impl<const S: u8, const R: usize> Deref for Segment<S, R> {
    type Target = SubSystem<SegmentControl<S, R>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const S: u8, const R: usize> DerefMut for Segment<S, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
