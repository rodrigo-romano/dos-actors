use matio_rs::MatFile;
use nalgebra as na;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use gmt_dos_clients_fem::{fem_io, DiscreteStateSpace, ExponentialMatrix, StateSpaceError};

use crate::builder::Include;

#[derive(Debug, thiserror::Error)]
pub enum FacesheetError {
    #[error("Failed to get Matlab ")]
    Matio(#[from] matio_rs::MatioError),
    #[error("Failed to get FEM Input/Output")]
    FEM(#[from] gmt_fem::FemError),
}

#[derive(Debug, Clone, Default)]
struct TransformMat {
    path: PathBuf,
    var_prefix: String,
}

/**
ASMS facesheet builder

The facesheet builder adds the following outputs to the FEM:
 * [`M2ASMFaceSheetFigure`](gmt_dos_clients_io::gmt_m2::asm::M2ASMFaceSheetFigure)
 * [`FaceSheetFigure<ID>`](gmt_dos_clients_io::gmt_m2::asm::segment::FaceSheetFigure)

The rigid body motions of the facesheet are removed per default.
If is not desirable to remove the rigid body motions of the facesheet,
the type parameter `R` can be set to `false`.

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
            ),
        )
        .build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```
 */
#[derive(Debug, Clone, Default)]
pub struct Facesheet<const R: bool> {
    filter_piston_tip_tip: bool,
    transforms_path: Option<TransformMat>,
    transforms: Option<Vec<na::DMatrix<f64>>>,
}

impl<const R: bool> Facesheet<R> {
    /// Creates a new [Facesheet] builder
    /// ```no_run
    /// # use gmt_dos_clients_servos::{asms_servo, AsmsServo, GmtServoMechanisms};
    /// # use gmt_fem::FEM;
    /// # const ACTUATOR_RATE: usize = 80; // 100Hz
    /// # let frequency = 8000_f64; // Hz
    /// # let fem = FEM::from_env()?;
    /// let gmt_servos =
    ///     GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
    ///         .asms_servo(
    ///             AsmsServo::new().facesheet(
    ///                 asms_servo::Facesheet::new()
    ///             ),
    ///         )
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Self {
        Default::default()
    }
    /// Removes the piston, tip and tilt components from the ASMS facesheets
    /// ```no_run
    /// # use gmt_dos_clients_servos::{asms_servo, AsmsServo, GmtServoMechanisms};
    /// # use gmt_fem::FEM;
    /// # const ACTUATOR_RATE: usize = 80; // 100Hz
    /// # let frequency = 8000_f64; // Hz
    /// # let fem = FEM::from_env()?;
    /// let gmt_servos =
    ///     GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
    ///         .asms_servo(
    ///             AsmsServo::new().facesheet(
    ///                 asms_servo::Facesheet::new()
    ///                     .filter_piston_tip_tilt()
    ///             ),
    ///         )
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn filter_piston_tip_tilt(mut self) -> Self {
        self.filter_piston_tip_tip = true;
        self
    }
    /// Sets the path to the file holding the matrix transform applied to the ASMS facesheets
    ///
    /// The file should be a MATLAB file with the variables: `var_#` where `#` stands
    /// for the segment number.
    /// ```no_run
    /// # use gmt_dos_clients_servos::{asms_servo, AsmsServo, GmtServoMechanisms};
    /// # use gmt_fem::FEM;
    /// # const ACTUATOR_RATE: usize = 80; // 100Hz
    /// # let frequency = 8000_f64; // Hz
    /// # let fem = FEM::from_env()?;
    /// let gmt_servos =
    ///     GmtServoMechanisms::<ACTUATOR_RATE, 1>::new(frequency, fem)
    ///         .asms_servo(
    ///             AsmsServo::new().facesheet(
    ///                 asms_servo::Facesheet::new()
    ///                     .transforms("KLmodesGS36p90.mat", "KL")
    ///             ),
    ///         )
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn transforms<P: AsRef<Path>>(mut self, path: P, var: impl ToString) -> Self {
        self.transforms_path = Some(TransformMat {
            path: path.as_ref().to_owned(),
            var_prefix: var.to_string(),
        });
        self
    }
    pub(crate) fn build<'a>(&'a mut self, fem: &gmt_fem::FEM) -> Result<(), FacesheetError> {
        self.transforms = match (self.transforms_path.as_ref(), self.filter_piston_tip_tip) {
            (None, true) => {
                println!("Filtering piston,tip and tilt from ASMS facesheets");
                let now = Instant::now();
                let ptt_free: Vec<_> = (0..7)
                    .into_par_iter()
                    .map(|i| {
                        let output_name = format!("M2_segment_{}_axial_d", i + 1);
                        // println!("Loading nodes from {output_name}");
                        let idx = Box::<dyn fem_io::GetOut>::try_from(output_name.clone())
                            .map(|x| x.position(&fem.outputs))?
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

                        Ok(p_mat)
                    })
                    .collect::<Result<Vec<_>, FacesheetError>>()?;
                println!(" done in {}ms", now.elapsed().as_millis());
                Some(ptt_free)
            }
            (None, false) => None,
            (Some(TransformMat { path, var_prefix }), true) => {
                let mat_file = MatFile::load(&path)?;
                println!("Loading the ASMS facesheet matrix transforms");
                let now = Instant::now();
                let kl_mat_trans: Vec<na::DMatrix<f64>> = (1..=7)
                    .map(|i| {
                        Ok(mat_file
                            .var(format!("{var_prefix}_{i}"))
                            .map(|mat: na::DMatrix<f64>| mat.transpose())?)
                    })
                    .collect::<Result<Vec<_>, FacesheetError>>()?;
                println!(" done in {}ms", now.elapsed().as_millis());
                println!("Filtering piston,tip and tilt from ASMS facesheets");
                let now = Instant::now();
                let ptt_free_kl_mat_trans: Vec<_> = kl_mat_trans
                    .into_par_iter()
                    .enumerate()
                    .map(|(i, kl_mat_trans)| {
                        let output_name = format!("M2_segment_{}_axial_d", i + 1);
                        // println!("Loading nodes from {output_name}");
                        let idx = Box::<dyn fem_io::GetOut>::try_from(output_name.clone())
                            .map(|x| x.position(&fem.outputs))?
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

                        Ok(kl_mat_trans * p_mat)
                    })
                    .collect::<Result<Vec<_>, FacesheetError>>()?;
                println!(" done in {}ms", now.elapsed().as_millis());
                Some(ptt_free_kl_mat_trans)
            }
            (Some(TransformMat { path, var_prefix }), false) => {
                let mat_file = MatFile::load(&path)?;
                println!("Loading the ASMS facesheet matrix transforms");
                let now = Instant::now();
                let kl_mat_trans: Vec<na::DMatrix<f64>> = (1..=7)
                    .map(|i| {
                        Ok(mat_file
                            .var(format!("{var_prefix}_{i}"))
                            .map(|mat: na::DMatrix<f64>| mat.transpose())?)
                    })
                    .collect::<Result<Vec<_>, FacesheetError>>()?;
                println!(" done in {}ms", now.elapsed().as_millis());
                Some(kl_mat_trans)
            }
        };
        Ok(())
    }
    pub(crate) fn transforms_view<'a>(&'a mut self) -> Option<Vec<na::DMatrixView<'a, f64>>> {
        self.transforms
            .as_ref()
            .map(|transforms| transforms.iter().map(|t| t.as_view()).collect())
    }
}

impl<'a, const R: bool> Include<'a, Facesheet<R>> for DiscreteStateSpace<'a, ExponentialMatrix> {
    fn including(self, facesheet: Option<&'a mut Facesheet<R>>) -> Result<Self, StateSpaceError> {
        let Some(facesheet) = facesheet else {
            return Ok(self);
        };
        let this = if let Some(transforms) = facesheet.transforms_view() {
            self.outs_with_by_name(
                (1..=7).map(|i| format!("M2_segment_{i}_axial_d")).collect(),
                transforms,
            )?
        } else {
            self.outs_by_name((1..=7).map(|i| format!("M2_segment_{i}_axial_d")).collect())?
        };
        Ok(if R { this.set_facesheet_nodes()? } else { this })
    }
}
