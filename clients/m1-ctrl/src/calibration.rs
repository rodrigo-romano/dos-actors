use gmt_fem::{fem_io, FEM};
use nalgebra as na;

#[derive(Debug, Clone, Default)]
pub struct Calibration {
    pub(crate) stiffness: f64,
    pub(crate) rbm_2_hp: Vec<M>,
    pub(crate) lc_2_cg: Vec<M>,
}

type M = nalgebra::Matrix6<f64>;

impl Calibration {
    pub fn new(fem: &mut FEM) -> Self {
        // Hardpoints stiffness
        log::info!("HARDPOINTS STIFFNESS");
        fem.switch_inputs(gmt_fem::Switch::Off, None)
            .switch_outputs(gmt_fem::Switch::Off, None);
        let Some(gain) =
        fem.switch_input::<fem_io::OSSHarpointDeltaF>(gmt_fem::Switch::On).and_then(|fem|
            fem.switch_output::<fem_io::OSSHardpointD>(gmt_fem::Switch::On))
            .and_then(|fem| fem.reduced_static_gain()) else {
                panic!(r#"failed to derive hardpoints stiffness, check input "OSSHarpointDeltaF" and output "OSSHardpointD""#)
            };
        let mut stiffness = 0f64;
        for i in 0..7 {
            let rows = gain.rows(i * 12, 12);
            let segment = rows.columns(i * 6, 6);
            let cell = segment.rows(0, 6);
            let face = segment.rows(6, 6);
            stiffness += (face - cell).diagonal().map(f64::recip).mean();
        }
        stiffness /= 7f64;

        // RBM2HP
        log::info!("RBM 2 HP");
        fem.switch_inputs(gmt_fem::Switch::Off, None)
            .switch_outputs(gmt_fem::Switch::Off, None);
        let Some(gain) =
        fem.switch_input::<fem_io::OSSHarpointDeltaF>(gmt_fem::Switch::On).and_then(|fem|
         fem.switch_output::<fem_io::OSSM1Lcl>(gmt_fem::Switch::On))
            .and_then(|fem| fem.reduced_static_gain()) else {
                panic!(r#"failed to derive hardpoints stiffness, check input "OSSHarpointDeltaF" and output "OSSM1Lcl""#)
            };
        let mut rbm_2_hp = vec![];
        for i in 0..7 {
            let rows = gain.rows(i * 6, 6);
            let segment = rows
                .columns(i * 6, 6)
                .try_inverse()
                .unwrap()
                .map(|x| x / stiffness);
            rbm_2_hp.push(na::Matrix6::from_column_slice(segment.as_slice()))
        }

        // LC2CG (include negative feedback)
        log::info!("LC 2 CG");
        fem.switch_inputs(gmt_fem::Switch::Off, None)
            .switch_outputs(gmt_fem::Switch::Off, None);
        let Some(gain) =
        fem.switch_input::<fem_io::OSSM1Lcl6F>(gmt_fem::Switch::On).and_then(|fem|
         fem.switch_output::<fem_io::OSSHardpointD>(gmt_fem::Switch::On))
            .and_then(|fem| fem.reduced_static_gain()) else {
                panic!(r#"failed to derive hardpoints stiffness, check input "OSSM1Lcl6F" and output "OSSHardpointD""#)
            };
        let mut lc_2_cg = vec![];
        for i in 0..7 {
            let rows = gain.rows(i * 12, 12);
            let segment = rows.columns(i * 6, 6);
            let cell = segment.rows(0, 6);
            let face = segment.rows(6, 6);
            let mat = (cell - face).try_inverse().unwrap().map(|x| x / stiffness);
            lc_2_cg.push(na::Matrix6::from_column_slice(mat.as_slice()));
        }

        fem.switch_inputs(gmt_fem::Switch::On, None)
            .switch_outputs(gmt_fem::Switch::On, None);

        Self {
            stiffness,
            rbm_2_hp,
            lc_2_cg,
        }
    }
}
