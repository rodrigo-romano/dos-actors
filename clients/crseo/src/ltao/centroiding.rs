use crate::OpticalModel;
use crseo::centroiding::CentroidingBuilder;
use crseo::imaging::ImagingBuilder;
use crseo::{Builder, Centroiding, Imaging};
use gmt_dos_clients_io::optics::{Dev, Frame, SensorData};
use interface::{Data, Read, Update, Write};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum CentroidsError {
    #[error("failed to build optical model")]
    Crseo(#[from] crseo::error::CrseoError),
}

pub type Result<T> = std::result::Result<T, CentroidsError>;

pub struct Centroids {
    pub(crate) reference: Centroiding,
    pub(crate) centroids: Centroiding,
    frame: Option<Arc<crseo::imaging::Frame>>,
}

unsafe impl Send for Centroids {}
unsafe impl Sync for Centroids {}

impl TryFrom<&ImagingBuilder> for Centroids {
    type Error = CentroidsError;

    fn try_from(value: &ImagingBuilder) -> Result<Self> {
        Ok(Self {
            reference: CentroidingBuilder::from(value).build()?,
            centroids: CentroidingBuilder::from(value).build()?,
            frame: None,
        })
    }
}

impl Centroids {
    pub fn setup(&mut self, optical_model: &mut OpticalModel<Imaging>) {
        optical_model.update();
        let imgr = optical_model.sensor.as_mut().unwrap();
        self.reference.process(&mut imgr.frame(), None);
        self.reference
            .valid_lenslets(Some(imgr.fluxlet_threshold), None);
        imgr.reset();
    }
    pub fn n_valid_lenslets(&self) -> usize {
        self.reference.n_valid_lenslet as usize
    }
}

impl Update for Centroids {
    fn update(&mut self) {
        self.centroids
            .process(self.frame.as_ref().unwrap(), Some(&self.reference));
    }
}

impl Read<Frame<Dev>> for Centroids {
    fn read(&mut self, data: Data<Frame<Dev>>) {
        self.frame = Some(data.into_arc())
    }
}

impl Write<SensorData> for Centroids {
    fn write(&mut self) -> Option<Data<SensorData>> {
        Some(
            self.centroids
                .grab()
                //.valids(Some(&self.reference.valid_lenslets))
                .centroids
                .iter()
                .map(|x| *x as f64)
                .collect::<Vec<_>>()
                .into(),
        )
    }
}