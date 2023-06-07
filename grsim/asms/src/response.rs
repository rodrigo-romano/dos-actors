use crate::if64;
use indicatif::{ParallelProgressIterator, ProgressBar};
use nalgebra::DMatrix;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, Mul};

type CMat = DMatrix<if64>;

/// Multi-input multi-output frequency response
///
/// Stores the MIMO systea as a matrix for a given frequency
#[derive(Debug, Serialize, Deserialize)]
pub struct MIMO {
    nu: f64,
    frequency_response: CMat,
}

impl From<(f64, CMat)> for MIMO {
    fn from((nu, frequency_response): (f64, CMat)) -> Self {
        MIMO {
            nu,
            frequency_response,
        }
    }
}

impl Mul<&CMat> for &MIMO {
    type Output = MIMO;

    fn mul(self, rhs: &CMat) -> Self::Output {
        (self.nu, &self.frequency_response * rhs).into()
    }
}

impl Mul<&MIMO> for &CMat {
    type Output = MIMO;

    fn mul(self, rhs: &MIMO) -> Self::Output {
        (rhs.nu, self * &rhs.frequency_response).into()
    }
}

impl Mul<&MIMO> for CMat {
    type Output = MIMO;

    fn mul(self, rhs: &MIMO) -> Self::Output {
        (rhs.nu, self * &rhs.frequency_response).into()
    }
}

impl MIMO {
    /// Returns the diagonal from the [MIMO] frequency response matrix
    pub fn diagonal(&self) -> Vec<if64> {
        self.frequency_response.diagonal().as_slice().to_vec()
    }
    /// Returns the singular values of the [MIMO] frequency response matrix
    pub fn singular_values(&self) -> Vec<f64> {
        self.frequency_response
            .clone()
            .svd(false, false)
            .singular_values
            .as_slice()
            .to_vec()
    }
}

/// Multi-input multi-output system frequency response
///
/// A system consists of [MIMO]s frequence response at multiple frequencies
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Sys(pub(crate) Vec<MIMO>);
impl Deref for Sys {
    type Target = Vec<MIMO>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<(Vec<f64>, Vec<CMat>)> for Sys {
    fn from((nu, frequency_response): (Vec<f64>, Vec<CMat>)) -> Self {
        Sys(nu
            .into_iter()
            .zip(frequency_response.into_iter())
            .map(|x| MIMO::from(x))
            .collect())
    }
}
impl FromIterator<MIMO> for Sys {
    fn from_iter<T: IntoIterator<Item = MIMO>>(iter: T) -> Self {
        Sys(iter.into_iter().collect())
    }
}
impl Mul<&CMat> for &Sys {
    type Output = Sys;

    fn mul(self, rhs: &CMat) -> Self::Output {
        let mimos: Vec<MIMO> = self.par_iter().progress().map(|mimo| mimo * rhs).collect();
        Sys(mimos)
    }
}

impl Mul<&CMat> for Sys {
    type Output = Sys;

    fn mul(self, rhs: &CMat) -> Self::Output {
        let mimos: Vec<MIMO> = self.par_iter().progress().map(|mimo| mimo * rhs).collect();
        Sys(mimos)
    }
}

impl Mul<&Sys> for &CMat {
    type Output = Sys;

    fn mul(self, rhs: &Sys) -> Self::Output {
        let mimos: Vec<MIMO> = rhs.par_iter().progress().map(|mimo| self * mimo).collect();
        Sys(mimos)
    }
}

impl Mul<&Sys> for CMat {
    type Output = Sys;

    fn mul(self, rhs: &Sys) -> Self::Output {
        // rhs.iter().map(|mimo| &self * mimo).collect()
        let mimos: Vec<MIMO> = rhs.par_iter().progress().map(|mimo| &self * mimo).collect();
        Sys(mimos)
    }
}

impl Sys {
    ///  Moves all the elements of other into self and comsuming other
    pub fn append(&mut self, mut other: Sys) {
        self.0.append(&mut other.0)
    }
    /// Returns the diagonals from all the [MIMO] systems
    pub fn diagonals(&self) -> Vec<Vec<if64>> {
        self.par_iter()
            .progress()
            .map(|mimo| mimo.diagonal())
            .collect()
    }
    /// Returns the singular values from all the [MIMO] systems
    pub fn singular_values(&self) -> Vec<Vec<f64>> {
        self.par_iter()
            .progress()
            .map(|mimo| mimo.singular_values())
            .collect()
    }

    /// Returns the [Sys]tem frequency vector
    pub fn frequencies(&self) -> Vec<f64> {
        self.iter().map(|tf| tf.nu).collect()
    }
    #[cfg(feature = "archive")]
    pub fn from_tarball<P: AsRef<std::path::Path>>(path: P) -> std::result::Result<Self, SysError> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tar::Archive;

        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        let gz_decoder = GzDecoder::new(reader);
        let mut archive = Archive::new(gz_decoder);

        let mut sys: Sys = Default::default();
        let pb = ProgressBar::new_spinner();
        for entry in archive.entries()? {
            let mut entry = entry?;
            pb.set_message(
                entry
                    .path()?
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
            pb.tick();
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;
            let mimo: Sys = bincode::deserialize(&data)?;
            sys.append(mimo);
        }
        sys.0.sort_by(|a, b| a.nu.partial_cmp(&b.nu).unwrap());
        Ok(sys)
    }
}

#[cfg(feature = "archive")]
#[derive(Debug, thiserror::Error)]
pub enum SysError {
    #[error(transparent)]
    File(#[from] std::io::Error),
    #[error(transparent)]
    Bincode(#[from] bincode::Error),
}
