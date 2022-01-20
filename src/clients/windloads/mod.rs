use std::time::Instant;

use crate::Client;
use parse_monitors::Monitors;

#[derive(Debug, thiserror::Error)]
pub enum WindLoadsError {
    #[error("loading the windloads failed")]
    Load(#[from] parse_monitors::MonitorsError),
}
pub type Result<T> = std::result::Result<T, WindLoadsError>;

/// List of  all the CFD wind loads
pub enum WindLoads {
    MirrorCovers,
}
impl WindLoads {
    /// Returns the names of the CFD monitors
    pub fn keys(self) -> Vec<String> {
        use WindLoads::*;
        match self {
            MirrorCovers => (1..=6)
                .map(|i| format!("M1cov{}", i))
                .chain((1..=6).map(|i| format!("M1covin{}", i)))
                .collect(),
        }
    }
}

/// [CfdLoads] builder
#[derive(Default)]
pub struct Builder {
    cfd_case: String,
    duration: Option<usize>,
    keys: Option<Vec<String>>,
}
impl Builder {
    /// Returns a [CfdLoads] [Builder]
    pub fn new<S: Into<String>>(cfd_case: S) -> Self {
        Self {
            cfd_case: cfd_case.into(),
            ..Default::default()
        }
    }
    /// Sets the wind loads time duration
    ///
    /// The duration is counted from the end of the recording
    pub fn duration(self, duration: usize) -> Self {
        Self {
            duration: Some(duration),
            ..self
        }
    }
    /// Sets the names of the CFD monitors
    pub fn keys(self, keys: Vec<String>) -> Self {
        Self {
            keys: Some(keys),
            ..self
        }
    }
    /// Returns a [CfdLoads] object
    pub fn build(self) -> Result<CfdLoads> {
        println!("Loading the CFD loads from {} ...", self.cfd_case);
        let now = Instant::now();
        let mut monitors = Monitors::loader::<String, 2021>(self.cfd_case).load()?;
        println!(" - data loaded in {}s", now.elapsed().as_secs());
        if let Some(duration) = self.duration {
            monitors.keep_last(duration).into_local();
        }
        let mut fm: Vec<Option<Vec<f64>>> = vec![];
        if let Some(keys) = self.keys {
            for i in 0..monitors.len() {
                for key in keys.iter() {
                    fm.push((&monitors.forces_and_moments[key][i]).into());
                }
            }
        } else {
            for i in 0..monitors.len() {
                for value in monitors.forces_and_moments.values() {
                    fm.push((&value[i]).into());
                }
            }
        }
        let data: Vec<f64> = fm.into_iter().filter_map(|x| x).flatten().collect();
        let n = data.len() / monitors.time.len();
        Ok(CfdLoads { data, n })
    }
}

/// The CFD loads
#[derive(Default, Debug)]
pub struct CfdLoads {
    data: Vec<f64>,
    n: usize,
}
impl CfdLoads {
    /// Creates a new [CfdLoads] object
    pub fn builder<S: Into<String>>(cfd_case: S) -> Builder {
        Builder::new(cfd_case)
    }
}

impl Client for CfdLoads {
    type I = ();
    type O = Vec<f64>;
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        if self.data.is_empty() {
            None
        } else {
            Some(vec![self.data.drain(..self.n).collect()])
        }
    }
}
