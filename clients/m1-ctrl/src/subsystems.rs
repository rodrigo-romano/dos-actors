mod segment;

use gmt_dos_actors::subsystem::{Built, SubSystem};
pub use segment::SegmentControl;

use crate::Calibration;

pub enum Segment<const S: u8, const R: usize> {}

pub(crate) type SegmentSubSystem<const S: u8, const R: usize> =
    SubSystem<SegmentControl<S, R>, 1, 1, Built>;

impl<const S: u8, const R: usize> Segment<S, R> {
    pub fn new(calibration: &Calibration) -> anyhow::Result<SegmentSubSystem<S, R>> {
        Ok(SubSystem::new(SegmentControl::<S, R>::new(calibration))
            .name(format!("M1S#{S}"))
            .build()?
            .flowchart())
    }
}

pub enum M1Assembly<const R: usize> {}

type M1S<const R: usize> = (
    SegmentSubSystem<1, R>,
    SegmentSubSystem<2, R>,
    SegmentSubSystem<3, R>,
    SegmentSubSystem<4, R>,
    SegmentSubSystem<5, R>,
    SegmentSubSystem<6, R>,
    SegmentSubSystem<7, R>,
);

impl<const R: usize> M1Assembly<R> {
    pub fn new(calibration: &Calibration) -> anyhow::Result<M1S<R>> {
        Ok((
            Segment::<1, R>::new(calibration)?,
            Segment::<2, R>::new(calibration)?,
            Segment::<3, R>::new(calibration)?,
            Segment::<4, R>::new(calibration)?,
            Segment::<5, R>::new(calibration)?,
            Segment::<6, R>::new(calibration)?,
            Segment::<7, R>::new(calibration)?,
        ))
    }
}
