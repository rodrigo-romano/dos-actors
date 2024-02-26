use std::{cell::RefCell, rc::Rc, sync::Arc};

use crseo::{
    wavefrontsensor::PhaseSensor, Atmosphere, CrseoError, FromBuilder, Gmt, SegmentWiseSensor,
    Source,
};
use gmt_dos_clients_domeseeing::{DomeSeeing, DomeSeeingError};
use gmt_dos_clients_io::{
    gmt_m1::{segment::RBM, M1ModeShapes, M1RigidBodyMotions},
    gmt_m2::{asm::segment::FaceSheetFigure, M2RigidBodyMotions},
    optics::{
        M2modes, SegmentPiston, SegmentTipTilt, SegmentWfe, SegmentWfeRms, Wavefront, WfeRms,
    },
};
use interface::{
    select::Selector, Data, Read, Size, TimerMarker, UniqueIdentifier, Units, Update, Write,
};

use crate::{DetectorFrame, OpticalModelBuilder};

#[derive(Debug, thiserror::Error)]
pub enum OpticalModelError {
    #[error(transparent)]
    CRSEO(#[from] CrseoError),
    #[error(transparent)]
    DomeSeeing(#[from] DomeSeeingError),
}

/// GMT optical model
///
/// ```no_run
/// use gmt_dos_clients_crseo::OpticalModel;
/// let optical_model_builder = OpticalModel::builder().build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct OpticalModel<T = PhaseSensor> {
    pub gmt: Gmt,
    pub src: Rc<RefCell<Source>>,
    pub atm: Option<Atmosphere>,
    pub dome_seeing: Option<DomeSeeing>,
    pub tau: f64,
    pub piston: Option<Arc<Vec<f64>>>,
    pub sensor: Option<T>,
}

impl Default for OpticalModel<PhaseSensor> {
    fn default() -> Self {
        <OpticalModelBuilder as Default>::default().build().unwrap()
    }
}

unsafe impl<T> Send for OpticalModel<T> {}
unsafe impl<T> Sync for OpticalModel<T> {}

impl<T> OpticalModel<T>
where
    T: FromBuilder,
    T::ComponentBuilder: Default,
{
    /// Return the [OpticalModelBuilder]
    pub fn builder() -> OpticalModelBuilder<T::ComponentBuilder> {
        Default::default()
    }
}

impl<T> Units for OpticalModel<T> {}
impl<T> Selector for OpticalModel<T> {}

impl<T> TimerMarker for OpticalModel<T> {}
impl<T> Update for OpticalModel<T>
where
    T: SegmentWiseSensor,
    OpticalModel<T>: Send + Sync,
{
    fn update(&mut self) {
        self.src.borrow_mut().through(&mut self.gmt).xpupil();
        if let Some(atm) = &mut self.atm {
            atm.secs += self.tau;
            self.src.borrow_mut().through(atm);
        }
        if let Some(dome_seeing) = &mut self.dome_seeing {
            self.src
                .borrow_mut()
                .add_same(dome_seeing.next().unwrap().as_slice());
        }
        if let Some(piston) = self.piston.as_deref() {
            self.src.borrow_mut().add_piston(piston.as_slice());
        }
        if let Some(sensor) = &mut self.sensor {
            sensor.propagate(&mut *self.src.borrow_mut())
        }
    }
}

// impl<T> Write<GuideStar> for OpticalModel<T> {
//     fn write(&mut self) -> Option<Data<GuideStar>> {
//         Some(Data::new(self.src.clone()))
//     }
// }

impl<T> Write<DetectorFrame> for OpticalModel<T>
where
    T: SegmentWiseSensor,
    DetectorFrame: UniqueIdentifier<DataType = crseo::Frame>,
{
    fn write(&mut self) -> Option<Data<DetectorFrame>> {
        self.sensor.as_mut().map(|sensor| {
            let frame = SegmentWiseSensor::frame(sensor);
            <T as crseo::WavefrontSensor>::reset(sensor);
            Data::new(frame)
        })
    }
}

impl<T: SegmentWiseSensor> Read<M2modes> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2modes>) {
        if 7 * self.gmt.m2_n_mode > data.len() {
            let augmented_data: Vec<_> = data
                .chunks(data.len() / 7)
                .flat_map(|data| {
                    let mut v = vec![0f64];
                    v.extend_from_slice(data);
                    v
                })
                .collect();
            assert_eq!(augmented_data.len(), self.gmt.m2_n_mode * 7);
            self.gmt.m2_modes(&augmented_data);
        } else {
            self.gmt.m2_modes(&data);
        }
    }
}
impl<T: SegmentWiseSensor, const ID: u8> Read<FaceSheetFigure<ID>> for OpticalModel<T> {
    fn read(&mut self, data: Data<FaceSheetFigure<ID>>) {
        self.gmt.m2_segment_modes(ID, &data);
    }
}
impl<T: SegmentWiseSensor> Read<M1RigidBodyMotions> for OpticalModel<T> {
    fn read(&mut self, data: Data<M1RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(id, v)| {
            let (t_xyz, r_xyz) = v.split_at(3);
            self.gmt.m1_segment_state((id + 1) as i32, t_xyz, r_xyz);
        });
    }
}
impl<T: SegmentWiseSensor> Read<M2RigidBodyMotions> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(id, v)| {
            let (t_xyz, r_xyz) = v.split_at(3);
            self.gmt.m2_segment_state((id + 1) as i32, t_xyz, r_xyz);
        });
    }
}
impl<T: SegmentWiseSensor, const ID: u8> Read<RBM<ID>> for OpticalModel<T> {
    fn read(&mut self, data: Data<RBM<ID>>) {
        let (t_xyz, r_xyz) = data.split_at(3);
        self.gmt.m1_segment_state(ID as i32, &t_xyz, &r_xyz);
    }
}
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

// #[derive(interface::UID)]
// #[uid(data = (Vec<f32>,Vec<bool>))]
pub enum GmtWavefront {}
impl UniqueIdentifier for GmtWavefront {
    type DataType = (Vec<f32>, Vec<bool>);
}

impl<T: SegmentWiseSensor> Write<GmtWavefront> for OpticalModel<T> {
    fn write(&mut self) -> Option<Data<GmtWavefront>> {
        let src = &mut *self.src.borrow_mut();
        let amplitude: Vec<_> = src.amplitude().into_iter().map(|a| a > 0.).collect();
        let phase = src.phase();
        let phase: Vec<_> = amplitude
            .iter()
            .zip(phase)
            .filter(|(&a, _)| a)
            .map(|(_, &p)| p)
            .collect();
        Some(Data::new((phase, amplitude)))
    }
}

impl<T: SegmentWiseSensor> Read<M1ModeShapes> for OpticalModel<T> {
    fn read(&mut self, data: Data<M1ModeShapes>) {
        self.gmt.m1_modes(&*data);
    }
}

impl<T: SegmentWiseSensor> Size<M1ModeShapes> for OpticalModel<T> {
    fn len(&self) -> usize {
        self.gmt.m1_n_mode * 7
    }
}
