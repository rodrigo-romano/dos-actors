use anyhow::{Context, Result};
use gmt_dos_clients_io::gmt_m2::asm::M2ASMVoiceCoilsMotion;
use interface::{Data, Read, UniqueIdentifier, Update, Write};
use matio_rs::MatFile;
use na::{DMatrix, DVector};
use std::{env, mem, path::Path, sync::Arc};

use nalgebra as na;

/// Voice coils displacements to rigid body motions
pub struct VoiceCoilToRbm {
    data: Arc<Vec<Arc<Vec<f64>>>>,
    vc_2_rbm: Vec<DMatrix<f64>>,
    y: Vec<f64>,
}

impl VoiceCoilToRbm {
    pub fn new() -> Result<Self> {
        let data_repo = env::var("DATA_REPO").context("`DATA_REPO` is not set")?;
        let mat_file = MatFile::load(Path::new(&data_repo).join("m2_vc_r.mat"))?;
        let mut vc_2_rbm = Vec::<DMatrix<f64>>::new();
        for i in 1..=7 {
            vc_2_rbm.push(mat_file.var(format!("m2_s{i}_vc_r"))?);
        }
        Ok(Self {
            data: Default::default(),
            vc_2_rbm,
            y: vec![0f64; 42],
        })
    }
}

impl Update for VoiceCoilToRbm {
    fn update(&mut self) {
        let _ = mem::replace(
            &mut self.y,
            self.data
                .iter()
                .zip(&self.vc_2_rbm)
                .map(|(data, vc_2_rbm)| -vc_2_rbm * DVector::from_column_slice(data.as_slice()))
                .flat_map(|x| x.as_slice().to_vec())
                .collect::<Vec<_>>(),
        );
    }
}

impl Read<M2ASMVoiceCoilsMotion> for VoiceCoilToRbm {
    fn read(&mut self, data: Data<M2ASMVoiceCoilsMotion>) {
        self.data = data.into_arc();
    }
}

impl<U: UniqueIdentifier<DataType = Vec<f64>>> Write<U> for VoiceCoilToRbm {
    fn write(&mut self) -> Option<Data<U>> {
        Some(self.y.clone().into())
    }
}
