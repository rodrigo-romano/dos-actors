use gmt_dos_clients_io::{
    gmt_m2::asm::M2ASMVoiceCoilsMotion,
    optics::{MaskedWavefront, Wavefront},
};
use gmt_lom::{LinearOpticalModelError, Loader, LoaderTrait};
use interface::Data;
use nalgebra as na;
use std::path::Path;

pub struct OpticalSensitivities<const N: usize> {
    sensitivity: gmt_lom::OpticalSensitivities<N>,
    data: Option<na::DMatrix<f64>>,
}

type Result<T> = std::result::Result<T, LinearOpticalModelError>;

impl<const N: usize> OpticalSensitivities<N> {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let filename = path.file_name().unwrap();
        let sensitivity = Loader::<gmt_lom::OpticalSensitivities<N>>::default()
            .path(path.parent().unwrap())
            .filename(filename.to_str().unwrap())
            .load()?;
        Ok(Self {
            sensitivity,
            data: None,
        })
    }
}

impl<const N: usize> interface::Update for OpticalSensitivities<N> {}

impl<const N: usize> interface::Read<M2ASMVoiceCoilsMotion> for OpticalSensitivities<N> {
    fn read(&mut self, data: Data<M2ASMVoiceCoilsMotion>) {
        let data: Vec<_> = data.into_arc().iter().flat_map(|x| x.to_vec()).collect();
        self.data = Some(na::DMatrix::from_vec(data.len(), 1, data));
    }
}

impl<const N: usize> interface::Write<Wavefront> for OpticalSensitivities<N> {
    fn write(&mut self) -> Option<Data<Wavefront>> {
        self.data
            .as_ref()
            .map(|data| Data::new(self.sensitivity.wavefront(data).into()))
    }
}

impl<const N: usize> interface::Write<MaskedWavefront> for OpticalSensitivities<N> {
    fn write(&mut self) -> Option<Data<MaskedWavefront>> {
        self.data
            .as_ref()
            .map(|data| Data::new(self.sensitivity.masked_wavefront(data).into()))
    }
}
