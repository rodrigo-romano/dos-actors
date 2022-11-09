use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
    time::Instant,
};

use nalgebra as na;
use serde::{Deserialize, Serialize};

mod bessel_knu;
pub use bessel_knu::fun as ku;

mod delta_opd;
pub use delta_opd::{
    DeltaOPD, DeltaOPDSet, DeltaOPDSetBuilder, DeltaOPDSubset, DeltaOpdParam, StructureFunction,
    StructureFunctionSample, StructureFunctionSubSample,
};

mod turbulence_model;
pub use turbulence_model::{a2r0, TurbulenceModel};

mod phase_map;
pub use phase_map::{Map, PhaseMap, PhaseMapBuilder};

#[derive(thiserror::Error)]
pub enum OpdError {
    #[error("cannot find CFD case")]
    CfdCaseNotFound(#[from] parse_monitors::cfd::CfdError),
    #[error("file not found")]
    OpdFileNotFound(#[from] std::io::Error),
    #[error("failed to read OPDs")]
    OpdRead(#[from] bincode::Error),
    #[error("failed to solve linear system of equations ({0})")]
    PolyFit(String),
    #[error("failed to build CRSEO object")]
    CRSEO(#[from] crseo::CrseoError),
}
fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
impl std::fmt::Debug for OpdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
pub type Result<T> = std::result::Result<T, OpdError>;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Opds {
    pub values: Vec<f64>,
    pub mask: Vec<bool>,
}
impl Opds {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let opds: Opds = bincode::deserialize_from(File::open(path.as_ref().join("opds.bin"))?)?;
        Ok(opds)
    }
    pub fn builder() -> OpdsBuilder {
        Default::default()
    }
}

#[derive(PartialEq, Default)]
pub enum Turbulence {
    Atmosphere,
    #[default]
    DomeSeeing,
}

pub struct OpdsBuilder {
    path: PathBuf,
    filename: String,
    atmosphere: Option<AtmosphereOpdsBuilder>,
}
impl Default for OpdsBuilder {
    fn default() -> Self {
        Self {
            path: Path::new("/fsx/CASES/zen30az000_OS7").into(),
            filename: "opds.bin".to_string(),
            atmosphere: None,
        }
    }
}
impl OpdsBuilder {
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = path.as_ref().into();
        self
    }
    pub fn filename(mut self, filename: String) -> Self {
        self.filename = filename;
        self
    }
    pub fn atmosphere(mut self, atmosphere: AtmosphereOpdsBuilder) -> Self {
        self.atmosphere = Some(atmosphere);
        self
    }
    pub fn build(self) -> Result<Opds> {
        if let Some(atmosphere) = self.atmosphere {
            println!("Computing atmospheric turbulence OPDs ...");
            let now = Instant::now();
            let (values, mask) = atmosphere.build()?;
            println!(" ... in {}ms", now.elapsed().as_millis());
            Ok(Opds { values, mask })
        } else {
            Ok(Opds::new(self.filename)?)
        }
    }
}

pub struct AtmosphereOpdsBuilder {
    n_xy: usize,
    delta: f64,
    n_sample: usize,
    outer_scale: Option<f64>,
    wind_speed: Option<f32>,
    mask: Option<Vec<bool>>,
}
impl Default for AtmosphereOpdsBuilder {
    fn default() -> Self {
        Self {
            n_xy: 104,
            n_sample: 1000,
            delta: 0.25,
            outer_scale: None,
            wind_speed: None,
            mask: None,
        }
    }
}
impl AtmosphereOpdsBuilder {
    pub fn n_sample(mut self, n_sample: usize) -> Self {
        self.n_sample = n_sample;
        self
    }
    pub fn outer_scale(mut self, outer_scale: f64) -> Self {
        self.outer_scale = Some(outer_scale);
        self
    }
    pub fn wind_speed(mut self, wind_speed: f64) -> Self {
        self.wind_speed = Some(wind_speed as f32);
        self
    }
    pub fn build(self) -> Result<(Vec<f64>, Vec<bool>)> {
        let n_xy = self.n_xy;
        let delta = self.delta;
        let diameter = delta * (n_xy - 1) as f64;
        let mask = self.mask;

        env::set_var(
            "GMT_MODES_PATH",
            "/home/ubuntu/projects/dos-actors/clients/domeseeing/analysis/projects/dos101/data",
        );

        let mut src = crseo::Builder::build(
            <crseo::Source as crseo::FromBuilder>::builder()
                .pupil_sampling(n_xy)
                .pupil_size(diameter),
        )?;
        let mut atm_builder = <crseo::Atmosphere as crseo::FromBuilder>::builder();
        if let Some(outer_scale) = self.outer_scale {
            atm_builder = atm_builder.oscale(outer_scale);
        }
        let mut atm = crseo::Builder::build(atm_builder.single_turbulence_layer(
            0.,
            self.wind_speed.clone(),
            None,
        ))?;
        println!(
            "Atmospheric turbulence with r0={:.3}cm and L0={:.3}m",
            atm.r0() * 1e2,
            atm.oscale
        );

        let n_xy2 = n_xy * n_xy;
        let mut xy: Vec<(f64, f64)> = Vec::with_capacity(n_xy2);
        if let Some(mask) = mask.clone() {
            let mut mask_iter = mask.iter();
            for i in 0..n_xy {
                for j in 0..n_xy {
                    if *mask_iter.next().unwrap() {
                        let x = (i as f64) * delta;
                        let y = (j as f64) * delta;
                        xy.push((x, y));
                    }
                }
            }
        } else {
            for i in 0..n_xy {
                for j in 0..n_xy {
                    let x = (i as f64) * delta;
                    let y = (j as f64) * delta;
                    xy.push((x, y));
                }
            }
        }

        let (x, y): (Vec<_>, Vec<_>) = xy.into_iter().unzip();
        let values: Vec<f64> = (0..self.n_sample)
            .flat_map(|i| {
                let t = if self.wind_speed.is_none() {
                    atm.reset();
                    0f64
                } else {
                    0.2 * i as f64
                };
                atm.get_phase_values(&mut src, x.as_slice(), y.as_slice(), t)
            })
            .collect();

        Ok((values, mask.unwrap_or(vec![true; n_xy2])))
    }
}

pub fn polyfit<T: na::RealField + Copy>(
    x_values: &[T],
    y_values: &[T],
    polynomial_degree: usize,
) -> Result<Vec<T>> {
    let number_of_columns = polynomial_degree + 1;
    let number_of_rows = x_values.len();
    let mut a = na::DMatrix::zeros(number_of_rows, number_of_columns);

    for (row, &x) in x_values.iter().enumerate() {
        // First column is always 1
        a[(row, 0)] = T::one();

        for col in 1..number_of_columns {
            a[(row, col)] = x.powf(na::convert(col as f64));
        }
    }

    let b = na::DVector::from_row_slice(y_values);

    let decomp = na::SVD::new(a, true, true);

    match decomp.solve(&b, na::convert(1e-18f64)) {
        Ok(mat) => Ok(mat.data.into()),
        Err(error) => Err(OpdError::PolyFit(error.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atmosphere() {
        let n_sample = 1;
        let builder = AtmosphereOpdsBuilder::default()
            .n_sample(n_sample)
            .outer_scale(50f64)
            .wind_speed(7f64);
        let opds = OpdsBuilder::default().atmosphere(builder).build().unwrap();
        let dopds = DeltaOPDSet::builder()
            .opds(std::rc::Rc::new(opds))
            .max_baseline(0.5f64)
            .n_baseline(1_000)
            .build()
            .unwrap();
        let mut sf: StructureFunctionSample = dopds
            .iter()
            .filter(|&x| *x == DeltaOpdParam::Time(0))
            .collect::<DeltaOPDSubset>()
            .into();
        let ac = sf.power_law_fit();
        println!("Power law: {ac:?}");
    }
}
