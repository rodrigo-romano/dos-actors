use crate::{
    ngao::DetectorFrame,
    sensors::{NoSensor, SensorPropagation},
    OpticalModelBuilder,
};
use crseo::{Atmosphere, FromBuilder, Gmt, SegmentWiseSensor, Source};
use gmt_dos_clients_io::{
    gmt_m1::{
        segment::{BendingModes, RBM},
        M1ModeShapes, M1RigidBodyMotions,
    },
    gmt_m2::{
        asm::{segment::AsmCommand, M2ASMAsmCommand},
        M2RigidBodyMotions,
    },
    optics::{M2modes, SegmentD7Piston},
};
use interface::{Data, Read, UniqueIdentifier, Update, Write};

pub mod builder;
mod imaging;
mod pyramid;
mod stats;

#[derive(Debug, thiserror::Error)]
pub enum OpticalModelError {
    #[error("failed to build optical model")]
    Crseo(#[from] crseo::error::CrseoError),
    #[error("atmosphere is set but not the sampling frequency")]
    AtmosphereWithoutSamplingFrequency,
}

/// GMT optical model
///
/// # Examples:
///
/// Build a optical model with the default [OpticalModelBuilder]
/// ```
/// use gmt_dos_clients_crseo::{OpticalModel, sensors::NoSensor};
///
/// let om = OpticalModel::<NoSensor>::builder().build()?;
/// # Ok::<(),Box<dyn std::error::Error>>(())
/// ```
pub struct OpticalModel<T = NoSensor> {
    pub(crate) gmt: Gmt,
    pub(crate) src: Source,
    pub(crate) atm: Option<Atmosphere>,
    pub(crate) sensor: Option<T>,
    pub(crate) tau: f64,
}

impl<T> OpticalModel<T> {
    /// Returns a mutable reference to the sensor
    pub fn sensor_mut(&mut self) -> Option<&mut T> {
        self.sensor.as_mut()
    }
    /// Returns an immutable reference to the sensor
    pub fn sensor(&self) -> Option<&T> {
        self.sensor.as_ref()
    }
}
unsafe impl<T> Send for OpticalModel<T> {}
unsafe impl<T> Sync for OpticalModel<T> {}

impl<T> OpticalModel<T>
where
    T: FromBuilder,
{
    pub fn builder() -> OpticalModelBuilder<<T as FromBuilder>::ComponentBuilder> {
        let OpticalModelBuilder {
            gmt,
            src,
            atm_builder,
            sampling_frequency,
            ..
        } = OpticalModelBuilder::<NoSensor>::default();
        OpticalModelBuilder {
            gmt,
            src,
            atm_builder,
            sensor: Some(T::builder()),
            sampling_frequency,
        }
    }
}

impl<T: SensorPropagation> Update for OpticalModel<T> {
    fn update(&mut self) {
        self.src.through(&mut self.gmt).xpupil();
        if let Some(atm) = &mut self.atm {
            atm.secs += self.tau;
            self.src.through(atm);
        }
        if let Some(sensor) = &mut self.sensor {
            sensor.propagate(&mut self.src);
        }
    }
}

impl<T: SensorPropagation, const SID: u8> Read<RBM<SID>> for OpticalModel<T> {
    fn read(&mut self, data: Data<RBM<SID>>) {
        self.gmt
            .m1_segment_state(SID as i32, &data[..3], &data[3..]);
    }
}

impl<T: SensorPropagation> Read<M1RigidBodyMotions> for OpticalModel<T> {
    fn read(&mut self, data: Data<M1RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(sid, data)| {
            self.gmt
                .m1_segment_state(1 + sid as i32, &data[..3], &data[3..]);
        });
    }
}

impl<T: SensorPropagation, const SID: u8> Read<BendingModes<SID>> for OpticalModel<T> {
    fn read(&mut self, data: Data<BendingModes<SID>>) {
        self.gmt.m1_segment_modes(SID, &data);
    }
}

impl<T: SensorPropagation> Read<M1ModeShapes> for OpticalModel<T> {
    fn read(&mut self, data: Data<M1ModeShapes>) {
        self.gmt.m1_modes(&data);
    }
}

impl<T: SensorPropagation> Read<M2RigidBodyMotions> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2RigidBodyMotions>) {
        data.chunks(6).enumerate().for_each(|(sid, data)| {
            self.gmt
                .m2_segment_state(1 + sid as i32, &data[..3], &data[3..]);
        });
    }
}

impl<T: SensorPropagation, const SID: u8> Read<AsmCommand<SID>> for OpticalModel<T> {
    fn read(&mut self, data: Data<AsmCommand<SID>>) {
        self.gmt.m2_segment_modes(SID, &data);
    }
}

impl<T: SensorPropagation> Read<M2ASMAsmCommand> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2ASMAsmCommand>) {
        self.gmt.m2_modes(&data);
    }
}

impl<T: SensorPropagation> Read<M2modes> for OpticalModel<T> {
    fn read(&mut self, data: Data<M2modes>) {
        if 7 * self.gmt.m2.n_mode > data.len() {
            let augmented_data: Vec<_> = data
                .chunks(data.len() / 7)
                .flat_map(|data| {
                    let mut v = vec![0f64];
                    v.extend_from_slice(data);
                    v
                })
                .collect();
            assert_eq!(augmented_data.len(), self.gmt.m2.n_mode * 7);
            self.gmt.m2_modes(&augmented_data);
        } else {
            self.gmt.m2_modes(&data);
        }
    }
}

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

impl<T: SegmentWiseSensor, const E: i32> Write<SegmentD7Piston<E>> for OpticalModel<T> {
    fn write(&mut self) -> Option<Data<SegmentD7Piston<E>>> {
        let data = self.src.segment_wfe();
        let p7 = data[6].0;
        // let data = &self.segment_wfe;
        Some(
            data.into_iter()
                .map(|(p, _)| (p - p7) * 10_f64.powi(-E))
                .collect::<Vec<_>>()
                .into(),
        )
    }
}
