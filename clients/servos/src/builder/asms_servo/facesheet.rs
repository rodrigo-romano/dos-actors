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
    pub fn build<'a>(&'a mut self, fem: &gmt_fem::FEM) -> Result<(), Box<dyn std::error::Error>> {
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

                        p_mat
                    })
                    .collect();
                println!(" done in {}ms", now.elapsed().as_millis());
                Some(ptt_free)
            }
            (None, false) => None,
            (Some(path), true) => {
                let mat_file = MatFile::load(&path)?;
                println!("Loading the ASMS facesheet matrix transforms");
                let now = Instant::now();
                let kl_mat_trans: Vec<na::DMatrix<f64>> = (1..=7)
                    .map(|i| mat_file.var(format!("KL_{i}")).unwrap())
                    .map(|mat: na::DMatrix<f64>| mat.transpose())
                    .collect();
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
            }
            (Some(path), false) => {
                let mat_file = MatFile::load(&path)?;
                println!("Loading the ASMS facesheet matrix transforms");
                let now = Instant::now();
                let kl_mat_trans: Vec<na::DMatrix<f64>> = (1..=7)
                    .map(|i| mat_file.var(format!("KL_{i}")).unwrap())
                    .map(|mat: na::DMatrix<f64>| mat.transpose())
                    .collect();
                println!(" done in {}ms", now.elapsed().as_millis());
                Some(kl_mat_trans)
            }
        };
        Ok(())
    }
    pub fn transforms_view<'a>(&'a mut self) -> Option<Vec<na::DMatrixView<'a, f64>>> {
        self.transforms
            .as_ref()
            .map(|transforms| transforms.iter().map(|t| t.as_view()).collect())
    }
}
