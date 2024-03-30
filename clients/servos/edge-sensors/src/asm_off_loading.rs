use anyhow::{Context, Result};
use gmt_dos_clients_io::gmt_m2::{asm::M2ASMReferenceBodyNodes, M2EdgeSensors};
use interface::{Data, Read, Update, Write};
use matio_rs::MatFile;
use std::{env, path::Path, sync::Arc};

use nalgebra as na;

use crate::EdgeSensorsAsRbms;

/// ASMS actuators to reference bodies off-load
pub struct AsmsOffLoading {
    // 36x36
    // rbm_2_mode: na::DMatrix<f64>,
    // 36x6
    r7_2_es: na::DMatrix<f64>,
    // 36x48
    es_2_r: na::DMatrix<f64>,
    // 42
    rbms: Arc<Vec<f64>>,
    // 42
    edge_sensors: Arc<Vec<f64>>,
    // 42
    data: Arc<Vec<f64>>,
}
impl AsmsOffLoading {
    pub fn new() -> Result<Self> {
        let data_repo = env::var("DATA_REPO").context("`DATA_REPO` is not set")?;
        //  * M2S7 RIGID-BODY MOTIONS TO EDGE SENSORS
        let r7_2_es: na::DMatrix<f64> = MatFile::load(Path::new(&data_repo).join("m2_r7_es.mat"))
            .context("Failed to read from m2_r7_es.mat")?
            .var("m2_r7_es")?;
        //  * EDGE SENSORS TO M2 RBMS
        let fem_var = env::var("FEM_REPO").expect("`FEM_REPO` is not set");
        let fem_path = Path::new(&fem_var);
        let es_2_r = MatFile::load(fem_path.join("m12_e_rs").join("m12_r_es.mat"))
            .context("Failed to read from m12_r_es.mat")?
            .var("m2_r_es")?;
        Ok(Self {
            r7_2_es,
            es_2_r,
            rbms: Default::default(),
            edge_sensors: Default::default(),
            data: Default::default(),
        })
    }
}
impl Update for AsmsOffLoading {
    fn update(&mut self) {
        let r7 = &self.rbms[36..];
        let es_from_r7 = &self.r7_2_es * na::DVector::from_column_slice(r7);
        // let rbm_2_mode =
        //     &self.rbm_2_mode * na::DVector::from_column_slice(&self.edge_sensors[..36]);
        let data: Vec<_> = self
            .edge_sensors
            .iter()
            .zip(es_from_r7.into_iter())
            .map(|(x, y)| x - y)
            .collect();
        let rbm = &self.es_2_r * na::DVector::from_column_slice(&data);
        let data: Vec<_> = rbm
            .into_iter()
            .map(|x| *x)
            .chain(r7.into_iter().map(|x| *x))
            .collect::<Vec<_>>();
        self.data = Arc::new(data);
    }
}
impl Read<M2ASMReferenceBodyNodes> for AsmsOffLoading {
    fn read(&mut self, data: Data<M2ASMReferenceBodyNodes>) {
        self.rbms = data.into_arc();
    }
}
impl Read<M2EdgeSensors> for AsmsOffLoading {
    fn read(&mut self, data: Data<M2EdgeSensors>) {
        self.edge_sensors = data.into_arc();
    }
}
impl Write<EdgeSensorsAsRbms> for AsmsOffLoading {
    fn write(&mut self) -> Option<Data<EdgeSensorsAsRbms>> {
        Some(self.data.clone().into())
    }
}
