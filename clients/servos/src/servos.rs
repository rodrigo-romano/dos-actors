use gmt_dos_actors::actor::PlainActor;
use gmt_dos_actors::{
    framework::network::{AddActorOutput, AddOuput, TryIntoInputs},
    prelude::Actor,
    system::{Sys, System},
};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::{
    gmt_fem::{
        inputs::{MCM2Lcl6F, MCM2SmHexF, OSSM1Lcl6F, CFD2021106F},
        outputs::{MCM2Lcl6D, MCM2SmHexD, OSSM1Lcl, MCM2RB6D},
    },
    gmt_m1::assembly,
    gmt_m2::{
        asm::{M2ASMFluidDampingForces, M2ASMVoiceCoilsForces, M2ASMVoiceCoilsMotion},
        M2PositionerForces, M2PositionerNodes,
    },
    mount::{MountEncoders, MountTorques},
};
use gmt_dos_clients_m1_ctrl::assembly::M1;
use gmt_dos_clients_m1_ctrl::Calibration;
use gmt_dos_clients_m2_ctrl::assembly::ASMS;
use gmt_dos_clients_m2_ctrl::positioner::AsmsPositioners;
use gmt_dos_clients_mount::Mount;

pub mod io;
pub mod traits;

#[derive(Clone)]
pub struct GmtServoMechanisms<'a, const M1_RATE: usize, const M2_RATE: usize = 1> {
    pub fem: Actor<DiscreteModalSolver<ExponentialMatrix>>,
    pub mount: Actor<Mount<'a>>,
    pub m1: Sys<M1<M1_RATE>>,
    pub m2_positioners: Actor<AsmsPositioners>,
    pub m2: Sys<ASMS<1>>,
}

impl<'a, const M1_RATE: usize, const M2_RATE: usize> GmtServoMechanisms<'static, M1_RATE, M2_RATE> {
    pub fn new(sim_sampling_frequency: f64, mut fem: gmt_fem::FEM) -> anyhow::Result<Self> {
        let sids: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7];
        let state_space = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem.clone())
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            //.max_eigen_frequency(75f64)
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
            .use_static_gain_compensation()
            .build()?;

        let m1_calibration = Calibration::new(&mut fem);

        let positioners = AsmsPositioners::from_fem(&mut fem)?;
        let asms = gmt_dos_clients_m2_ctrl::ASMS::<1>::from_fem(&mut fem, None)?;

        println!("{fem}");

        let mount = Mount::new();
        let m1 = gmt_dos_clients_m1_ctrl::M1::<M1_RATE>::new(&m1_calibration)?;

        Ok(Self {
            fem: state_space.into(),
            mount: mount.into(),
            m1,
            m2_positioners: positioners.into(),
            m2: asms,
        })
    }
}

impl<const M1_RATE: usize, const M2_RATE: usize> System
    for GmtServoMechanisms<'static, M1_RATE, M2_RATE>
{
    fn name(&self) -> String {
        format!("GMT Servo-Mechanisms (M1@{M1_RATE})")
    }
    fn build(&mut self) -> anyhow::Result<&mut Self> {
        self.mount
            .add_output()
            .build::<MountTorques>()
            .into_input(&mut self.fem)?;
        self.fem
            .add_output()
            .bootstrap()
            .build::<MountEncoders>()
            .into_input(&mut self.mount)?;

        self.m1
            .add_output()
            .build::<assembly::M1HardpointsForces>()
            .into_input(&mut self.fem)?;
        self.fem
            .add_output()
            .bootstrap()
            .build::<assembly::M1HardpointsMotion>()
            .into_input(&mut self.m1)?;

        self.m1
            .add_output()
            .build::<assembly::M1ActuatorAppliedForces>()
            .into_input(&mut self.fem)?;

        self.m2_positioners
            .add_output()
            .build::<M2PositionerForces>()
            .into_input(&mut self.fem)?;
        self.fem
            .add_output()
            .bootstrap()
            .build::<M2PositionerNodes>()
            .into_input(&mut self.m2_positioners)?;

        self.m2
            .add_output()
            .build::<M2ASMVoiceCoilsForces>()
            .into_input(&mut self.fem)?;
        self.m2
            .add_output()
            .build::<M2ASMFluidDampingForces>()
            .into_input(&mut self.fem)?;
        self.fem
            .add_output()
            .bootstrap()
            .build::<M2ASMVoiceCoilsMotion>()
            .into_input(&mut self.m2)?;

        Ok(self)
    }
    fn plain(&self) -> gmt_dos_actors::actor::PlainActor {
        // self.segments.iter().for_each(|segment| segment.flowchart());
        // self.flowchart();
        let mut plain = PlainActor::default();
        plain.client = self.name();
        plain.inputs_rate = 1;
        plain.outputs_rate = 1;
        plain.inputs = PlainActor::from(&self.fem).inputs;
        plain.outputs = PlainActor::from(&self.fem).outputs;
        plain
    }
}
