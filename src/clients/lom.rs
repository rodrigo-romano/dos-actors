/// Linear Optical Model client
use crate::{
    io::{Data, Read, Write},
    Update,
};
use lom::LOM;
use std::{convert::AsMut, sync::Arc};

impl Update for LOM {}

#[cfg(feature = "fem")]
impl Read<Vec<f64>, fem::fem_io::OSSM1Lcl> for LOM {
    fn read(&mut self, data: Arc<Data<Vec<f64>, fem::fem_io::OSSM1Lcl>>) {
        self.rbm
            .as_mut()
            .column_mut(0)
            .iter_mut()
            .take(42)
            .zip(&**data)
            .for_each(|(rbm, val)| *rbm = *val);
    }
}

#[cfg(feature = "fem")]
impl Read<Vec<f64>, fem::fem_io::MCM2Lcl6D> for LOM {
    fn read(&mut self, data: Arc<Data<Vec<f64>, fem::fem_io::MCM2Lcl6D>>) {
        self.rbm
            .as_mut()
            .column_mut(0)
            .iter_mut()
            .skip(42)
            .zip(&**data)
            .for_each(|(rbm, val)| *rbm = *val);
    }
}

pub enum TipTilt {}
impl Write<Vec<f64>, TipTilt> for LOM {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, TipTilt>>> {
        Some(Arc::new(Data::new((*self.tiptilt()).clone())))
    }
}
pub enum SegmentTipTilt {}
impl Write<Vec<f64>, SegmentTipTilt> for LOM {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, SegmentTipTilt>>> {
        Some(Arc::new(Data::new((*self.segment_tiptilt()).clone())))
    }
}
#[cfg(feature = "fsm")]
impl Write<Vec<f64>, crate::clients::fsm::TTFB> for LOM {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, crate::clients::fsm::TTFB>>> {
        Some(Arc::new(Data::new((*self.segment_tiptilt()).clone())))
    }
}
pub enum SegmentPiston {}
impl Write<Vec<f64>, SegmentPiston> for LOM {
    fn write(&mut self) -> Option<Arc<Data<Vec<f64>, SegmentPiston>>> {
        Some(Arc::new(Data::new((*self.segment_tiptilt()).clone())))
    }
}
