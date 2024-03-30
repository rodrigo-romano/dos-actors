use anyhow::{Context, Result};
use interface::{Data, Read, UniqueIdentifier, Update, Write};
use matio_rs::MatFile;
use na::{DMatrix, DVector};
use std::{env, mem, path::Path, sync::Arc};

use nalgebra as na;

use crate::N_ACTUATOR;

/// Rigid body motions to facesheet displacements
pub struct RbmToShell {
    data: Arc<Vec<f64>>,
    rbm_2_shell: Vec<DMatrix<f64>>,
    y: Vec<f64>,
}

impl RbmToShell {
    pub fn new() -> Result<Self> {
        let data_repo = env::var("DATA_REPO").context("`DATA_REPO` is not set")?;
        let mat_file = MatFile::load(Path::new(&data_repo).join("rbm_2_faceheet.mat"))?;
        let mut rbm_2_shell = Vec::<DMatrix<f64>>::new();
        for i in 1..=7 {
            rbm_2_shell.push(mat_file.var(format!("m2_s{i}_rbm_2_shell"))?);
        }
        Ok(Self {
            data: Default::default(),
            rbm_2_shell,
            y: vec![0f64; N_ACTUATOR * 7],
        })
    }
}

impl Update for RbmToShell {
    fn update(&mut self) {
        let _ = mem::replace(
            &mut self.y,
            self.data
                .chunks(6)
                .zip(&self.rbm_2_shell)
                .map(|(data, rbm_2_shell)| rbm_2_shell * DVector::from_column_slice(data))
                .flat_map(|x| x.as_slice().to_vec())
                .collect::<Vec<_>>(),
        );
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Read<U> for RbmToShell {
    fn read(&mut self, data: Data<U>) {
        self.data = data.into_arc();
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for RbmToShell {
    fn write(&mut self) -> Option<Data<U>> {
        Some(self.y.clone().into())
    }
}
