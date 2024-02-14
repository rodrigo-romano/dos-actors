use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix};
use gmt_dos_clients_io::gmt_fem::{
    inputs::{MCM2Lcl6F, MCM2SmHexF, OSSM1Lcl6F, CFD2021106F},
    outputs::{MCM2Lcl6D, MCM2SmHexD, OSSM1Lcl, MCM2RB6D},
};
use gmt_dos_clients_m1_ctrl::Calibration;
use gmt_dos_clients_m2_ctrl::positioner::AsmsPositioners;
use gmt_dos_clients_mount::Mount;

use crate::servos::GmtServoMechanisms;

/// ASMS builder
#[derive(Debug, Clone, Default)]
pub struct AsmsServo {
    facesheet: asms_servo::Facesheet,
}

impl AsmsServo {
    /// Creates a new ASMS builder
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the ASMS [Facesheet](asms_servo::Facesheet) builder
    pub fn facesheet(mut self, facesheet: asms_servo::Facesheet) -> Self {
        self.facesheet = facesheet;
        self
    }
}

/**
Repository for the ASMS component builders

## Example

```no_run
use gmt_dos_clients_servos::{asms_servo, AsmsServo, GmtServoMechanisms};
use gmt_fem::FEM;

const ACTUATOR_RATE: usize = 80; // 100Hz

let frequency = 8000_f64; // Hz
let fem = FEM::from_env()?;

let gmt_servos =
    GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
        .asms_servo(
            AsmsServo::new().facesheet(
                asms_servo::Facesheet::new()
                    .filter_piston_tip_tilt()
                    .transforms("KLmodesGS36p90.mat"),
            ),
        )
        .build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
*/
pub mod asms_servo {
    use matio_rs::MatFile;
    use nalgebra as na;
    use rayon::prelude::*;
    use std::{
        path::{Path, PathBuf},
        time::Instant,
    };

    use gmt_dos_clients_fem::fem_io;

    /// ASMS facesheet builder
    #[derive(Debug, Clone, Default)]
    pub struct Facesheet {
        filter_piston_tip_tip: bool,
        transforms_path: Option<PathBuf>,
        transforms: Option<Vec<na::DMatrix<f64>>>,
    }

    impl Facesheet {
        /// Creates a mew [Facesheet] builder
        pub fn new() -> Self {
            Default::default()
        }
        /// Removes the piston, tip and tilt components from the ASMS facesheets
        pub fn filter_piston_tip_tilt(mut self) -> Self {
            self.filter_piston_tip_tip = true;
            self
        }
        /// Sets the path to the file holding the matrix transform applied to the ASMS facesheets
        pub fn transforms<P: AsRef<Path>>(mut self, path: P) -> Self {
            self.transforms_path = Some(path.as_ref().to_owned());
            self
        }
        pub(crate) fn get_transforms<'a>(
            &'a mut self,
            fem: &gmt_fem::FEM,
        ) -> Option<Vec<na::DMatrixView<'a, f64>>> {
            let path = self.transforms_path.as_ref()?;
            let mat_file = MatFile::load(&path).ok()?;
            println!("Loading the ASMS facesheet matrix transforms");
            let now = Instant::now();
            let kl_mat_trans: Vec<na::DMatrix<f64>> = (1..=7)
                .map(|i| mat_file.var(format!("KL_{i}")).unwrap())
                .map(|mat: na::DMatrix<f64>| mat.transpose())
                .collect();
            println!(" done in {}ms", now.elapsed().as_millis());
            self.transforms = if self.filter_piston_tip_tip {
                println!("Filtering piston,tip and tilt from ASMS facesheets");
                let now = Instant::now();
                let ptt_free_kl_mat_trans: Vec<_> = kl_mat_trans
                    .into_par_iter()
                    .enumerate()
                    .map(|(i, kl_mat_trans)| {
                        let output_name = format!("M2_segment_{}_axial_d", i + 1);
                        // println!("Loading nodes from {output_name}");
                        let idx = Box::<dyn fem_io::GetOut>::try_from(output_name.clone())
                            .map(|x| x.position(&fem.outputs))
                            .ok()
                            .unwrap()
                            .expect(&format!(
                                "failed to find the index of the output: {output_name}"
                            ));
                        let xyz = fem.outputs[idx]
                            .as_ref()
                            .map(|i| i.get_by(|i| i.properties.location.clone()))
                            .expect(&format!(
                                "failed to read nodes locations from {output_name}"
                            ));
                        let (x, y): (Vec<_>, Vec<_>) =
                            xyz.into_iter().map(|xyz| (xyz[0], xyz[1])).unzip();
                        let mut ones = na::DVector::<f64>::zeros(675);
                        ones.fill(1f64);
                        let x_vec = na::DVector::<f64>::from_row_slice(&x);
                        let y_vec = na::DVector::<f64>::from_row_slice(&y);
                        let t_mat = na::DMatrix::<f64>::from_columns(&[ones, x_vec, y_vec]);
                        let p_mat = na::DMatrix::<f64>::identity(675, 675)
                            - &t_mat * t_mat.clone().pseudo_inverse(0f64).unwrap();

                        kl_mat_trans * p_mat
                    })
                    .collect();
                println!(" done in {}ms", now.elapsed().as_millis());
                Some(ptt_free_kl_mat_trans)
            } else {
                Some(kl_mat_trans)
            };
            self.transforms
                .as_ref()
                .map(|transforms| transforms.iter().map(|t| t.as_view()).collect())
        }
    }
}

/// [GmtServoMechanisms](crate::GmtServoMechanisms) builder
#[derive(Debug, Clone, Default)]
pub struct ServosBuilder<const M1_RATE: usize, const M2_RATE: usize> {
    pub(crate) sim_sampling_frequency: f64,
    pub(crate) fem: gmt_fem::FEM,
    pub(crate) asms_servo: AsmsServo,
}

impl<const M1_RATE: usize, const M2_RATE: usize> ServosBuilder<M1_RATE, M2_RATE> {
    /// Sets the [ASMS](AsmsServo) builder
    pub fn asms_servo(mut self, asms_servo: AsmsServo) -> Self {
        self.asms_servo = asms_servo;
        self
    }
}

impl<'a, const M1_RATE: usize, const M2_RATE: usize> TryFrom<ServosBuilder<M1_RATE, M2_RATE>>
    for GmtServoMechanisms<'static, M1_RATE, M2_RATE>
{
    type Error = anyhow::Error;

    fn try_from(mut builder: ServosBuilder<M1_RATE, M2_RATE>) -> Result<Self, Self::Error> {
        let mut fem = builder.fem;

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
        let state_space_builder = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem.clone())
            .sampling(builder.sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
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
            .use_static_gain_compensation();

        let state_space_builder =
            if let Some(transforms) = builder.asms_servo.facesheet.get_transforms(&fem) {
                state_space_builder.outs_with_by_name(
                    (1..=7).map(|i| format!("M2_segment_{i}_axial_d")).collect(),
                    transforms,
                )?
            } else {
                state_space_builder
            };

        let state_space = state_space_builder.build()?;

        Ok(Self {
            fem: (state_space, "GMT Structural\nDynamic Model").into(),
            mount: (mount, "Mount\nController").into(),
            m1,
            m2_positioners: (positioners, "M2 Positioners\nController").into(),
            m2: asms,
        })
    }
}
