//! CFD wind loads client implementation
//!

use geotrans::{SegmentTrait, Transform};
use interface::filing::Codec;
use parse_monitors::Vector;
use serde::{Deserialize, Serialize};
use std::fmt;

mod actors_interface;

#[derive(Debug, thiserror::Error)]
pub enum WindLoadsError {
    #[error("loading the windloads failed")]
    Load(#[from] parse_monitors::MonitorsError),
    #[error("coordinates transformation failed")]
    Coordinates(#[from] geotrans::Error),
}
pub type Result<T> = std::result::Result<T, WindLoadsError>;

const MAX_DURATION: usize = 400;

mod windloads;
pub use windloads::WindLoads;

mod builder;
pub use builder::{Builder, CS, M1S, M2S};

impl Builder<ZOH> {
    /// Returns a [CfdLoads] [Builder]
    pub fn zoh<C: Into<String>>(cfd_case: C) -> Self {
        Self {
            cfd_case: cfd_case.into(),
            upsampling: ZOH(20),
            ..Default::default()
        }
    }
}
impl Builder<FOH> {
    /// Returns a [CfdLoads] [Builder]
    pub fn foh<C: Into<String>>(cfd_case: C, upsampling: usize) -> Self {
        Self {
            cfd_case: cfd_case.into(),
            upsampling: FOH::new(upsampling / 20),
            ..Default::default()
        }
    }
}

/// Zero-order hold wind loads interpolation
///
/// Staircase interpolation between 2 CFD timestamps
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ZOH(usize);

/// First-order hold wind loads interpolation
///
/// Linear interpolation between 2 CFD timestamps
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FOH {
    rate: usize,
    i: usize,
    u: f64,
}
impl FOH {
    /// Creates a new first-order hold wind loads interpolator
    pub fn new(rate: usize) -> Self {
        Self {
            rate,
            ..Default::default()
        }
    }
    pub fn update(&mut self, step: usize) {
        self.i = step / self.rate;
        self.u = (step - self.i * self.rate) as f64 / self.rate as f64;
    }
    /// Interpolates linearly between 2 samples
    pub fn sample(&self, x: &[f64], n: usize) -> Option<Vec<f64>> {
        if let (Some(y0), Some(y1)) = (x.chunks(n).nth(self.i), x.chunks(n).nth(self.i + 1)) {
            Some(
                y0.iter()
                    .zip(y1.iter())
                    .map(|(y0, y1)| (y1 - y0) * self.u + y0)
                    .collect(),
            )
        } else {
            None
        }
    }
}
/// The CFD loads
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CfdLoads<S> {
    oss: Option<Vec<f64>>,
    m1: Option<Vec<f64>>,
    m2: Option<Vec<f64>>,
    nodes: Option<Vec<(String, CS)>>,
    n_fm: usize,
    step: usize,
    upsampling: S,
    max_step: usize,
}
impl CfdLoads<ZOH> {
    /// Creates a new [CfdLoads] object
    pub fn zoh<C: Into<String>>(cfd_case: C) -> Builder<ZOH> {
        Builder::zoh(cfd_case)
    }
}
impl CfdLoads<FOH> {
    /// Creates a new [CfdLoads] object
    pub fn foh<C: Into<String>>(cfd_case: C, upsampling: usize) -> Builder<FOH> {
        Builder::foh(cfd_case, upsampling)
    }
}

impl<S: Serialize + for<'de> Deserialize<'de>> Codec for CfdLoads<S> {}

impl<S> CfdLoads<S> {
    pub fn oss_mean(&self) -> Option<Vec<f64>> {
        self.oss.as_ref().map(|oss| {
            let n_step = (oss.len() / self.n_fm) as f64;
            oss.chunks(self.n_fm)
                .fold(vec![0f64; self.n_fm], |mut a, x| {
                    a.iter_mut().zip(x.iter()).for_each(|(a, x)| *a += x);
                    a
                })
                .into_iter()
                .map(|x| x / n_step)
                .collect::<Vec<f64>>()
        })
    }
    pub fn m1_mean(&self) -> Option<Vec<f64>> {
        self.m1.as_ref().map(|oss| {
            let n_step = (oss.len() / 42) as f64;
            oss.chunks(42)
                .fold(vec![0f64; 42], |mut a, x| {
                    a.iter_mut().zip(x.iter()).for_each(|(a, x)| *a += x);
                    a
                })
                .into_iter()
                .map(|x| x / n_step)
                .collect::<Vec<f64>>()
        })
    }
    pub fn stop_after(&mut self, max_step: usize) -> &mut Self {
        self.max_step = max_step;
        self
    }
    pub fn start_from(&mut self, step: usize) -> &mut Self {
        self.max_step = usize::MAX;
        self.step = step + 1;
        self
    }
}
impl<S> fmt::Display for CfdLoads<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(oss) = self.oss_mean() {
            writeln!(f, "CFD loads in OSS ({}):", oss.len() / 6)?;
            for (oss, (key, loc)) in oss.chunks(6).zip(
                self.nodes
                    .as_ref()
                    .expect("CFD loads locations missing")
                    .iter(),
            ) {
                if let CS::OSS(loc) = loc {
                    writeln!(
                        f,
                        " - {:<20} @ {:>5.1?}m : <{:>6.0?}>N <{:>6.0?}>N.m",
                        key,
                        loc,
                        &oss[..3],
                        &oss[3..]
                    )?;
                }
            }
        }
        if let Some(oss) = self.m1_mean() {
            writeln!(f, "CFD loads in M1 local:")?;
            let mut force = Vector::zero();
            let mut moment = Vector::zero();
            for (i, oss) in oss.chunks(6).enumerate() {
                writeln!(
                    f,
                    " - M1S{:} : <{:>6.0?}>N <{:>6.0?}>N.m",
                    i + 1,
                    &oss[..3],
                    &oss[3..]
                )?;
                let u: Vector = (&oss[..3])
                    .to_vec()
                    .vtov(M1S::new(i as i32 + 1))
                    .unwrap()
                    .into();
                let t: [f64; 3] = M1S::new(i as i32 + 1).unwrap().translation().into();
                let r: Vector = t.into();
                let mu = r.cross(&u).unwrap();
                force = force + u;
                let u: Vector = (&oss[3..])
                    .to_vec()
                    .vtov(M1S::new(i as i32 + 1))
                    .unwrap()
                    .into();
                moment = moment + u + mu;
            }
            let u: Option<Vec<f64>> = force.into();
            writeln!(f, " - sum mean forces (OSS) : {:6.0?}N", u.unwrap())?;
            let v: Option<Vec<f64>> = moment.into();
            writeln!(f, " - sum mean moments (OSS): {:6.0?}N.m", v.unwrap())?;
        }
        Ok(())
    }
}
