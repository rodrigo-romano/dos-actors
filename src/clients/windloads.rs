//! CFD wind loads [Client](crate::Client) implementation

use std::time::Instant;

use super::Client;
use parse_monitors::Monitors;

#[derive(Debug, thiserror::Error)]
pub enum WindLoadsError {
    #[error("loading the windloads failed")]
    Load(#[from] parse_monitors::MonitorsError),
    #[error("coordinates transformation failed")]
    Coordinates(#[from] geotrans::Error),
}
pub type Result<T> = std::result::Result<T, WindLoadsError>;

/// List of  all the CFD wind loads
#[derive(Debug)]
pub enum WindLoads {
    MirrorCovers,
    M1Segments,
    M1Cell,
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
            M1Segments => (1..=7).map(|i| format!("M1_{i}")).collect(),
            M1Cell => vec![String::from("M1cell")],
        }
    }
    /// Returns a pattern to match against the FEM CFD_202110_6F input
    pub fn fem(self) -> Vec<String> {
        use WindLoads::*;
        match self {
            MirrorCovers => vec![String::from("mirror cover")],
            M1Segments => (1..=7).map(|i| format!("M1-S{i} unit")).collect(),
            M1Cell => vec![String::from("M1 cells walls")],
        }
    }
}

#[derive(Debug)]
pub enum CS {
    OSS(Vec<f64>),
    M1S(i32),
}

/// [CfdLoads] builder
#[derive(Default, Debug)]
pub struct Builder {
    cfd_case: String,
    duration: Option<usize>,
    keys: Option<Vec<String>>,
    locations: Option<Vec<CS>>,
    nodes: Option<Vec<(String, CS)>>,
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

    /// Sets the nodes [x,y,z] coordinates where the loads are applied
    pub fn nodes(self, keys: Vec<String>, locations: Vec<CS>) -> Self {
        let nodes: Vec<_> = keys
            .into_iter()
            .zip(locations.into_iter())
            .map(|(x, y)| (x, y))
            .collect();
        Self {
            nodes: Some(nodes),
            ..self
        }
    }
    /// Requests M1 segments loads
    pub fn m1_segments(self) -> Self {
        let m1_nodes: Vec<_> = WindLoads::M1Segments
            .keys()
            .into_iter()
            .zip((1..=7).map(|i| CS::M1S(i)))
            .map(|(x, y)| (x, y))
            .collect();
        if let Some(mut nodes) = self.nodes {
            nodes.extend(m1_nodes.into_iter());
            Self {
                nodes: Some(nodes),
                ..self
            }
        } else {
            Self {
                nodes: Some(m1_nodes),
                ..self
            }
        }
    }
    /// Returns a [CfdLoads] object
    pub fn build(self) -> Result<CfdLoads> {
        println!("Loading the CFD loads from {} ...", self.cfd_case);
        let now = Instant::now();
        let mut monitors = Monitors::loader::<String, 2021>(self.cfd_case).load()?;
        println!(" - data loaded in {}s", now.elapsed().as_secs());
        if let Some(duration) = self.duration {
            monitors.keep_last(duration); //.into_local();
        }
        let mut fm: Option<Vec<Option<Vec<f64>>>> = None;
        let mut m1_fm: Option<Vec<Option<Vec<f64>>>> = None;
        if let Some(nodes) = self.nodes {
            for i in 0..monitors.len() {
                for (key, location) in nodes.iter() {
                    let exertion = monitors
                        .forces_and_moments
                        .get_mut(key)
                        .expect(&format!("{key} not found in CFD loads"));
                    match location {
                        CS::OSS(loc) => {
                            exertion[i].into_local(loc.into());
                            if let Some(fm) = fm.as_mut() {
                                fm.push((&exertion[i]).into());
                            } else {
                                fm = Some(vec![(&exertion[i]).into()]);
                            }
                        }
                        CS::M1S(j) => {
                            use geotrans::{Segment, SegmentTrait, Transform, M1};
                            type M1S = Segment<M1>;
                            let t: [f64; 3] = M1S::new(*j)?.translation().into();
                            exertion[i].into_local(t.into());
                            if let (Some(f), Some(m)) = (
                                Into::<Option<[f64; 3]>>::into(&exertion[i].force),
                                Into::<Option<[f64; 3]>>::into(&exertion[i].moment),
                            ) {
                                exertion[i].force = f.fro(M1S::new(*j))?.into();
                                exertion[i].moment = m.fro(M1S::new(*j))?.into();
                            };
                            if let Some(m1_fm) = m1_fm.as_mut() {
                                m1_fm.push((&exertion[i]).into());
                            } else {
                                m1_fm = Some(vec![(&exertion[i]).into()]);
                            }
                        }
                    };
                }
            }
        } else {
            for i in 0..monitors.len() {
                for exertion in monitors.forces_and_moments.values() {
                    if let Some(fm) = fm.as_mut() {
                        fm.push((&exertion[i]).into());
                    } else {
                        fm = Some(vec![(&exertion[i]).into()]);
                    }
                }
            }
        }
        let data: Option<Vec<f64>> = if let Some(fm) = fm {
            Some(fm.into_iter().filter_map(|x| x).flatten().collect())
        } else {
            None
        };
        let m1_loads: Option<Vec<f64>> = if let Some(fm) = m1_fm {
            Some(fm.into_iter().filter_map(|x| x).flatten().collect())
        } else {
            None
        };
        let n = data
            .as_ref()
            .map_or(m1_loads.as_ref().map_or(0, |x| x.len()), |x| x.len())
            / monitors.time.len();
        Ok(CfdLoads { data, m1_loads, n })
    }
}

/// The CFD loads
#[derive(Default, Debug)]
pub struct CfdLoads {
    data: Option<Vec<f64>>,
    m1_loads: Option<Vec<f64>>,
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
        match (self.data.as_mut(), self.m1_loads.as_mut()) {
            (Some(data), Some(m1_loads)) => {
                if data.is_empty() || m1_loads.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![
                        data.drain(..self.n).collect(),
                        m1_loads.drain(..42).collect(),
                    ])
                }
            }
            (Some(data), None) => {
                if data.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![data.drain(..self.n).collect()])
                }
            }
            (None, Some(m1_loads)) => {
                if m1_loads.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![m1_loads.drain(..self.n).collect()])
                }
            }
            (None, None) => None,
        }
    }
}
