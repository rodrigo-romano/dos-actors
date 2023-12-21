use gmt_dos_actors::{
    framework::model::Check,
    prelude::*,
    subsystem::{gateway, BuildSystem, GetField},
};
use gmt_dos_clients::Sampler;
use gmt_dos_clients_io::gmt_m1::segment::{
    ActuatorAppliedForces, ActuatorCommandForces, BarycentricForce, HardpointsForces,
    HardpointsMotion, RBM,
};

use crate::{Actuators, Calibration, Hardpoints, LoadCells};

#[derive(Clone)]
pub struct SegmentControl<const S: u8, const R: usize> {
    pub hardpoints: Actor<Hardpoints>,
    pub loadcells: Actor<LoadCells, 1, R>,
    pub actuators: Actor<Actuators<S>, R, 1>,
    pub sampler: Actor<Sampler<Vec<f64>, ActuatorCommandForces<S>>, 1, R>,
}

impl<const S: u8, const R: usize> GetField for SegmentControl<S, R> {
    fn get_field(&self, idx: usize) -> Option<&dyn Check> {
        match idx {
            0 => Some(&self.hardpoints as &dyn Check),
            1 => Some(&self.loadcells as &dyn Check),
            2 => Some(&self.actuators as &dyn Check),
            3 => Some(&self.sampler as &dyn Check),
            _ => None,
        }
    }
}

impl<const S: u8, const R: usize> From<SegmentControl<S, R>> for Model<Unknown> {
    fn from(s: SegmentControl<S, R>) -> Self {
        model!(s.hardpoints, s.loadcells, s.actuators, s.sampler)
    }
}

impl<const S: u8, const R: usize> SegmentControl<S, R> {
    pub fn new(calibration: &Calibration) -> Self {
        assert!(
            S > 0 && S < 8,
            "expect segment # in the range [1,7], found {S}"
        );
        let idx = (S - 1) as usize;

        let Calibration {
            stiffness,
            rbm_2_hp,
            lc_2_cg,
        } = calibration;

        let hardpoints: Actor<_> = (
            Hardpoints::new(*stiffness, rbm_2_hp[idx]),
            format!(
                "M1S{S}>
                    Hardpoints"
            ),
        )
            .into();

        let loadcells: Actor<_, 1, R> = (
            LoadCells::new(*stiffness, lc_2_cg[idx]),
            format!(
                "M1S{S}
                    Loadcells"
            ),
        )
            .into();

        let actuators: Actor<_, R, 1> = (
            Actuators::<S>::new(),
            format!(
                "M1S{S}
                    Actuators"
            ),
        )
            .into();

        Self {
            hardpoints,
            loadcells,
            actuators,
            sampler: Sampler::default().into(),
        }
    }
}

impl<const S: u8, const R: usize> gateway::Gateways for SegmentControl<S, R> {
    type DataType = Vec<f64>;

    const N_IN: usize = 3;

    const N_OUT: usize = 2;
}

/* // Local aliases for inputs & outputs
//  * RBM
#[derive(UID)]
#[alias(name = gmt_m1::segment::RBM<S>, client = Hardpoints, traits = Read )]
pub struct RBM<const S: u8> {}
//  * ActuatorCommandForces
#[derive(UID)]
#[alias(name = gmt_m1::segment::ActuatorCommandForces<S>, client = Actuators<S>, traits = Read )]
pub struct ActuatorCommandForces<const S: u8> {}
//  * HardpointsForces
#[derive(UID)]
#[alias(name = gmt_m1::segment::HardpointsForces<S>, client = Hardpoints, traits = Write )]
#[alias(name = gmt_m1::segment::HardpointsForces<S>, client = LoadCells, traits = Read )]
pub struct HardpointsForces<const S: u8> {}

//  * HardpointsForces
#[derive(UID)]
#[alias(name = gmt_m1::segment::ActuatorAppliedForces<S>, client = Actuators<S>, traits = Write )]
pub struct ActuatorAppliedForces<const S: u8> {}

//  * RBM
#[derive(UID)]
#[alias(name = gmt_m1::segment::HardpointsMotion<S>, client = LoadCells, traits = Read )]
pub struct HardpointsMotion<const S: u8> {}

// Mapping gateways data indices to inputs & output
//  * In[0] -> RBM<S>
impl<const S: u8> gateway::In for RBM<S> {
    const IDX: usize = 0;
}
//  * In[1] -> ActuatorCommandForces<S>
impl<const S: u8> gateway::In for ActuatorCommandForces<S> {
    const IDX: usize = 1;
}
//  * In[2] -> HardpointsMotion<S>
impl<const S: u8> gateway::In for HardpointsMotion<S> {
    const IDX: usize = 2;
}
//  * Out[0] -> HardpointsForces<S>
impl<const S: u8> gateway::Out for HardpointsForces<S> {
    const IDX: usize = 0;
}
//  * Out[1] -> ActuatorAppliedForces<S>
impl<const S: u8> gateway::Out for ActuatorAppliedForces<S> {
    const IDX: usize = 1;
} */

impl<const S: u8, const R: usize> BuildSystem<SegmentControl<S, R>> for SegmentControl<S, R> {
    fn build(
        &mut self,
        gateway_in: &mut Actor<gateway::WayIn<SegmentControl<S, R>>, 1, 1>,
        gateway_out: &mut Actor<gateway::WayOut<SegmentControl<S, R>>, 1, 1>,
    ) -> anyhow::Result<()> {
        gateway_in
            .add_output()
            .build::<RBM<S>>()
            .into_input(&mut self.hardpoints)?;

        gateway_in
            .add_output()
            .build::<ActuatorCommandForces<S>>()
            .into_input(&mut self.sampler)?;

        self.sampler
            .add_output()
            .build::<ActuatorCommandForces<S>>()
            .into_input(&mut self.actuators)?;

        self.hardpoints
            .add_output()
            .multiplex(2)
            .build::<HardpointsForces<S>>()
            .into_input(&mut self.loadcells)
            .into_input(gateway_out)?;

        self.loadcells
            .add_output()
            // .bootstrap()
            .build::<BarycentricForce<S>>()
            .into_input(&mut self.actuators)?;

        self.actuators
            .add_output()
            .build::<ActuatorAppliedForces<S>>()
            .into_input(gateway_out)?;

        gateway_in
            .add_output()
            // .bootstrap()
            .build::<HardpointsMotion<S>>()
            .into_input(&mut self.loadcells)?;

        Ok(())
    }
}
