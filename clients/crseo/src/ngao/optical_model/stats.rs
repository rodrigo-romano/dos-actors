use crseo::SegmentWiseSensor;
use gmt_dos_clients_io::optics::{
    SegmentPiston, SegmentTipTilt, SegmentWfe, SegmentWfeRms, Wavefront, WfeRms,
};
use interface::{Data, Read, Size, Write};

use crate::OpticalModel;

impl<T: SegmentWiseSensor, const E: i32> Size<WfeRms<E>> for OpticalModel<T> {
    fn len(&self) -> usize {
        self.src.borrow().size as usize
    }
}

impl<T: SegmentWiseSensor, const E: i32> Write<WfeRms<E>> for OpticalModel<T> {
    fn write(&mut self) -> Option<Data<WfeRms<E>>> {
        Some(
            match E {
                0 => self.src.borrow_mut().wfe_rms(),
                exp => self.src.borrow_mut().wfe_rms_10e(exp),
            }
            .into(),
        )
    }
}

impl<T: SegmentWiseSensor, const E: i32> Size<SegmentWfeRms<E>> for OpticalModel<T> {
    fn len(&self) -> usize {
        (self.src.borrow().size as usize) * 7
    }
}
impl<T: SegmentWiseSensor, const E: i32> Write<SegmentWfeRms<E>> for OpticalModel<T> {
    fn write(&mut self) -> Option<Data<SegmentWfeRms<E>>> {
        Some(
            match E {
                0 => self.src.borrow_mut().segment_wfe_rms(),
                exp => self.src.borrow_mut().segment_wfe_rms_10e(exp),
            }
            .into(),
        )
    }
}

impl<T: SegmentWiseSensor, const E: i32> Size<SegmentPiston<E>> for OpticalModel<T> {
    fn len(&self) -> usize {
        (self.src.borrow().size as usize) * 7
    }
}
impl<T: SegmentWiseSensor, const E: i32> Write<SegmentPiston<E>> for OpticalModel<T> {
    fn write(&mut self) -> Option<Data<SegmentPiston<E>>> {
        Some(
            match E {
                0 => self.src.borrow_mut().segment_piston(),
                exp => self.src.borrow_mut().segment_piston_10e(exp),
            }
            .into(),
        )
    }
}
impl<T: SegmentWiseSensor> Read<SegmentPiston> for OpticalModel<T> {
    fn read(&mut self, data: Data<SegmentPiston>) {
        self.piston = Some(data.into_arc());
    }
}

impl<T: SegmentWiseSensor> Size<SegmentWfe> for OpticalModel<T> {
    fn len(&self) -> usize {
        (self.src.borrow().size as usize) * 7
    }
}
impl<T: SegmentWiseSensor> Write<SegmentWfe> for OpticalModel<T> {
    fn write(&mut self) -> Option<Data<SegmentWfe>> {
        let src = &mut *self.src.borrow_mut();
        Some(Data::new(src.segment_wfe()))
    }
}

impl<T: SegmentWiseSensor> Size<SegmentTipTilt> for OpticalModel<T> {
    fn len(&self) -> usize {
        (self.src.borrow().size as usize) * 7 * 2
    }
}
impl<T: SegmentWiseSensor> Write<SegmentTipTilt> for OpticalModel<T> {
    fn write(&mut self) -> Option<Data<SegmentTipTilt>> {
        let src = &mut *self.src.borrow_mut();
        Some(Data::new(src.segment_gradients()))
    }
}

impl<T: SegmentWiseSensor> Size<Wavefront> for OpticalModel<T> {
    fn len(&self) -> usize {
        self.src.borrow().pupil_sampling().pow(2)
    }
}
impl<T: SegmentWiseSensor> Write<Wavefront> for OpticalModel<T> {
    fn write(&mut self) -> Option<Data<Wavefront>> {
        let src = &mut *self.src.borrow_mut();
        Some(Data::new(
            src.phase().into_iter().map(|x| *x as f64).collect(),
        ))
    }
}
