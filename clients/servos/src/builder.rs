use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix, StateSpaceError};
use gmt_dos_clients_io::gmt_fem::{
    inputs::{MCM2Lcl6F, MCM2SmHexF, OSSM1Lcl6F, CFD2021106F},
    outputs::{MCM2Lcl6D, MCM2SmHexD, OSSM1Lcl, MCM2RB6D},
};
use gmt_dos_clients_m1_ctrl::Calibration;
use gmt_dos_clients_m2_ctrl::positioner::AsmsPositioners;
use gmt_dos_clients_mount::Mount;

use crate::servos::GmtServoMechanisms;

pub mod asms_servo;
pub use asms_servo::AsmsServo;
//mod windloads;

/// [GmtServoMechanisms](crate::GmtServoMechanisms) builder
#[derive(Debug, Clone, Default)]
pub struct ServosBuilder<const M1_RATE: usize, const M2_RATE: usize> {
    pub(crate) sim_sampling_frequency: f64,
    pub(crate) fem: gmt_fem::FEM,
    pub(crate) asms_servo: Option<AsmsServo>,
}

impl<const M1_RATE: usize, const M2_RATE: usize> ServosBuilder<M1_RATE, M2_RATE> {
    /// Sets the [ASMS](AsmsServo) builder
    pub fn asms_servo(mut self, asms_servo: AsmsServo) -> Self {
        self.asms_servo = Some(asms_servo);
        self
    }
}

pub trait Include<'a, C> {
    fn including(self, component: Option<&'a mut C>) -> Result<Self, StateSpaceError>
    where
        Self: 'a + Sized;
}

impl<'a, const M1_RATE: usize, const M2_RATE: usize> TryFrom<ServosBuilder<M1_RATE, M2_RATE>>
    for GmtServoMechanisms<'static, M1_RATE, M2_RATE>
{
    type Error = anyhow::Error;

    fn try_from(mut builder: ServosBuilder<M1_RATE, M2_RATE>) -> Result<Self, Self::Error> {
        let mut fem = builder.fem;

        if let Some(asms_servo) = builder.asms_servo.as_mut() {
            asms_servo.build(&fem)?;
        }

        let mount = Mount::new();

        log::info!("Calibrating M1");
        let m1_calibration = Calibration::new(&mut fem);
        let m1 = gmt_dos_clients_m1_ctrl::M1::<M1_RATE>::new(&m1_calibration)?;

        log::info!("Calibrating ASMS positioners");
        let positioners = AsmsPositioners::from_fem(&mut fem)?;
        log::info!("Calibrating ASMS");
        let asms = gmt_dos_clients_m2_ctrl::ASMS::<1>::from_fem(&mut fem, None)?;

        log::info!("Building structural state space model");
        let sids: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7];
        let state_space = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem.clone())
            .sampling(builder.sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            .use_static_gain_compensation()
            .including_mount()
            .including_m1(Some(sids.clone()))?
            .including_asms(Some(sids.clone()), None, None)?
            .ins::<CFD2021106F>()
            .ins::<OSSM1Lcl6F>()
            .ins::<MCM2Lcl6F>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .ins::<MCM2SmHexF>()
            .outs::<MCM2SmHexD>()
            .outs::<MCM2RB6D>()
            .including(builder.asms_servo.as_mut())?
            .build()?;

        Ok(Self {
            fem: (state_space, "GMT Structural\nDynamic Model").into(),
            mount: (mount, "Mount\nController").into(),
            m1,
            m2_positioners: (positioners, "M2 Positioners\nController").into(),
            m2: asms,
        })
    }
}
