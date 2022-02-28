//! CFD wind loads [Client](crate::Client) implementation

use crate::{
    io::{Data, Read, Write},
    Update,
};
use geotrans::{Segment, SegmentTrait, Transform, M1, M2};
use parse_monitors::{Exertion, Monitors, Vector};
use std::{fmt, time::Instant};

#[derive(Debug, thiserror::Error)]
pub enum WindLoadsError {
    #[error("loading the windloads failed")]
    Load(#[from] parse_monitors::MonitorsError),
    #[error("coordinates transformation failed")]
    Coordinates(#[from] geotrans::Error),
}
pub type Result<T> = std::result::Result<T, WindLoadsError>;

/// List of  all the CFD wind loads
#[derive(Debug, Clone)]
pub enum WindLoads {
    TopEnd,
    M2Segments,
    M2Baffle,
    Trusses,
    M1Baffle,
    MirrorCovers,
    LaserGuideStars,
    CRings,
    GIR,
    //LPA,
    Platforms,
    M1Segments,
}
impl WindLoads {
    /// Returns the names of the CFD monitors
    pub fn keys(&self) -> Vec<String> {
        use WindLoads::*;
        match self {
            MirrorCovers => (1..=6)
                .map(|i| format!("M1cov{}", i))
                .chain((1..=6).map(|i| format!("M1covin{}", i)))
                .collect(),
            M1Segments => (1..=7).map(|i| format!("M1_{i}")).collect(),
            M2Segments => (1..=7).map(|i| format!("M2seg{i}")).collect(),
            TopEnd => vec![String::from("Topend")],
            M2Baffle => vec![String::from("M2Baffle")],
            Trusses => (1..=3)
                .map(|i| format!("Tup{i}"))
                .chain((1..=3).map(|i| format!("Tbot{i}")))
                .chain((1..=3).map(|i| format!("arm{i}")))
                .collect(),
            M1Baffle => vec![String::from("M1Baffle")],
            //LPA => vec![String::from("M1level")],
            LaserGuideStars => (1..=3).map(|i| format!("LGSS{i}")).collect(),
            CRings => [
                "CringL",
                "CringR",
                "Cring_strL",
                "Cring_strR",
                "Cring_strF",
                "Cring_strB",
            ]
            .into_iter()
            .map(|x| x.into())
            .collect(),
            GIR => vec!["GIR".into()],
            Platforms => vec!["platform".into()],
        }
    }
    /// Returns a pattern to match against the FEM CFD_202110_6F input
    pub fn fem(&self) -> Vec<String> {
        use WindLoads::*;
        match self {
            MirrorCovers => vec![String::from("mirror cover")],
            M1Segments => (1..=7).map(|i| format!("M1-S{i} unit")).collect(),
            M2Segments => (1..=7).map(|i| format!("M2 cell {i}.")).collect(),
            TopEnd => vec![String::from("Top-End")],
            M2Baffle => vec![String::from("M2 baffle unit")],
            Trusses => ["Upper truss", "Lower truss", "Focus Assembly Arm"]
                .into_iter()
                .map(|x| x.into())
                .collect(),
            M1Baffle => vec![String::from("Baffle protruding")],
            //LPA => vec![String::from("LPA")],
            LaserGuideStars => vec![String::from("Laser Guide Star")],
            CRings => [
                "C-Ring under M1 segments 5 and 6",
                "C-Ring under M1 segments 2 and 3",
                "outside of C-Ring below M1 cells 5 and 6",
                "outside of C-Ring below M1 cells 2 and 3",
                "between C-Rings below M1 cell 4",
                "between C-Rings below M1 cell 1",
            ]
            .into_iter()
            .map(|x| x.into())
            .collect(),
            GIR => vec!["GIR".into()],
            Platforms => vec!["Instrument, OSS mid-level, and Auxiliary Platforms".into()],
        }
    }
}

#[derive(Debug)]
pub enum CS {
    OSS(Vec<f64>),
    M1S(i32),
    M2S(i32),
}

type M1S = Segment<M1>;
type M2S = Segment<M2>;

/// [CfdLoads] builder
#[derive(Default, Debug)]
pub struct Builder<S> {
    cfd_case: String,
    duration: Option<f64>,
    nodes: Option<Vec<(String, CS)>>,
    upsampling_frequency: S,
}
impl<S: Default> Builder<S> {
    /// Sets the wind loads time duration
    ///
    /// The duration is counted from the end of the recording
    pub fn duration(self, duration: f64) -> Self {
        Self {
            duration: Some(duration),
            ..self
        }
    }
    /// Sets the nodes [x,y,z] coordinates where the loads are applied
    pub fn nodes(self, keys: Vec<String>, locations: Vec<CS>) -> Self {
        assert!(
            keys.len() == locations.len(),
            "the number of wind loads node locations ({}) do not match the number of keys ({})",
            locations.len(),
            keys.len()
        );
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
    /// Requests M2 segments loads
    pub fn m2_segments(self) -> Self {
        let m2_nodes: Vec<_> = WindLoads::M2Segments
            .keys()
            .into_iter()
            .zip((1..=7).map(|i| CS::M2S(i)))
            .map(|(x, y)| (x, y))
            .collect();
        if let Some(mut nodes) = self.nodes {
            nodes.extend(m2_nodes.into_iter());
            Self {
                nodes: Some(nodes),
                ..self
            }
        } else {
            Self {
                nodes: Some(m2_nodes),
                ..self
            }
        }
    }
}
impl<S> Builder<S> {
    /// Returns a [CfdLoads] object
    pub fn build(self) -> Result<CfdLoads<S>> {
        println!("Loading the CFD loads from {} ...", self.cfd_case);
        let now = Instant::now();
        let mut monitors = Monitors::loader::<String, 2021>(self.cfd_case).load()?;

        let fm = monitors.forces_and_moments.remove("Cabs").unwrap();
        monitors
            .forces_and_moments
            .get_mut("platform")
            .unwrap()
            .iter_mut()
            .zip(fm.into_iter())
            .for_each(|(p, c)| {
                let u = p.clone();
                *p = &u + &c;
            });
        let fm = monitors.forces_and_moments.remove("cabletrays").unwrap();
        monitors
            .forces_and_moments
            .get_mut("platform")
            .unwrap()
            .iter_mut()
            .zip(fm.into_iter())
            .for_each(|(p, c)| {
                let u = p.clone();
                *p = &u + &c;
            });

        println!(" - data loaded in {}s", now.elapsed().as_secs());
        if let Some(duration) = self.duration {
            monitors.keep_last(duration.ceil() as usize); //.into_local();
        }
        let mut fm: Option<Vec<Option<Vec<f64>>>> = None;
        let mut m1_fm: Option<Vec<Option<Vec<f64>>>> = None;
        let mut m2_fm: Option<Vec<Option<Vec<f64>>>> = None;
        let mut total_exertion: Vec<Exertion> = vec![];
        if let Some(ref nodes) = self.nodes {
            for i in 0..monitors.len() {
                total_exertion.push(Exertion {
                    force: Vector::zero(),
                    moment: Vector::zero(),
                    cop: None,
                });
                for (key, location) in nodes.iter() {
                    let mut m1_cell = monitors
                        .forces_and_moments
                        .get_mut("M1cell")
                        .expect("M1cell not found in CFD loads")
                        .clone();
                    let exertion = monitors
                        .forces_and_moments
                        .get_mut(key)
                        .expect(&format!("{key} not found in CFD loads"));
                    let u = total_exertion[i].clone();
                    total_exertion[i] = &u + &exertion[i].clone();
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
                            if *j < 2 {
                                let u = total_exertion[i].clone();
                                total_exertion[i] = &u + &m1_cell[i].clone();
                            }
                            let t: [f64; 3] = M1S::new(*j)?.translation().into();
                            exertion[i].into_local(t.into());
                            if *j < 7 {
                                m1_cell[i].into_local(t.into());
                                if let Some(m1_cell) = &m1_cell[i] / 6f64 {
                                    let v = &exertion[i] + &m1_cell;
                                    exertion[i] = v;
                                }
                            }
                            if let (Some(f), Some(m)) = (
                                Into::<Option<[f64; 3]>>::into(&exertion[i].force),
                                Into::<Option<[f64; 3]>>::into(&exertion[i].moment),
                            ) {
                                exertion[i].force = f.vfrov(M1S::new(*j))?.into();
                                exertion[i].moment = m.vfrov(M1S::new(*j))?.into();
                            };
                            if let Some(m1_fm) = m1_fm.as_mut() {
                                m1_fm.push((&exertion[i]).into());
                            } else {
                                m1_fm = Some(vec![(&exertion[i]).into()]);
                            }
                        }
                        CS::M2S(j) => {
                            let t: [f64; 3] = M2S::new(*j)?.translation().into();
                            exertion[i].into_local(t.into());
                            if let (Some(f), Some(m)) = (
                                Into::<Option<[f64; 3]>>::into(&exertion[i].force),
                                Into::<Option<[f64; 3]>>::into(&exertion[i].moment),
                            ) {
                                exertion[i].force = f.vfrov(M2S::new(*j))?.into();
                                exertion[i].moment = m.vfrov(M2S::new(*j))?.into();
                            };
                            if let Some(m2_fm) = m2_fm.as_mut() {
                                m2_fm.push((&exertion[i]).into());
                            } else {
                                m2_fm = Some(vec![(&exertion[i]).into()]);
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

        let n = total_exertion.len() as f64;
        let force_mean = (total_exertion
            .iter()
            .fold(Vector::zero(), |a, e| a + e.force.clone())
            / n)
            .unwrap();
        let mut force_std = total_exertion
            .iter()
            .map(|e| (e.force.clone() - force_mean.clone()).unwrap())
            .map(|v| {
                let a: Option<Vec<f64>> = v.into();
                let a = a.unwrap();
                vec![a[0] * a[0], a[1] * a[1], a[2] * a[2]]
            })
            .fold(vec![0f64; 3], |mut a, e| {
                a.iter_mut().zip(e.iter()).for_each(|(a, e)| {
                    *a += e;
                });
                a
            });
        force_std.iter_mut().for_each(|x| *x = (*x / n).sqrt());
        println!(
            " OSS force: mean = {:.0?}N ; std = {:.0?}N",
            force_mean, force_std
        );
        let moment_mean = (total_exertion
            .iter()
            .fold(Vector::zero(), |a, e| a + e.moment.clone())
            / n)
            .unwrap();
        let mut moment_std = total_exertion
            .iter()
            .map(|e| (e.moment.clone() - moment_mean.clone()).unwrap())
            .map(|v| {
                let a: Option<Vec<f64>> = v.into();
                let a = a.unwrap();
                vec![a[0] * a[0], a[1] * a[1], a[2] * a[2]]
            })
            .fold(vec![0f64; 3], |mut a, e| {
                a.iter_mut().zip(e.iter()).for_each(|(a, e)| {
                    *a += e;
                });
                a
            });
        moment_std.iter_mut().for_each(|x| *x = (*x / n).sqrt());
        println!(
            " OSS moment: mean = {:.0?}N.m ; std = {:.0?}N.m",
            moment_mean, moment_std
        );

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
        let m2_loads: Option<Vec<f64>> = if let Some(fm) = m2_fm {
            Some(fm.into_iter().filter_map(|x| x).flatten().collect())
        } else {
            None
        };
        let n = data
            .as_ref()
            .map_or(m1_loads.as_ref().map_or(0, |x| x.len()), |x| x.len())
            / monitors.time.len();
        Ok(CfdLoads {
            oss: data,
            m1: m1_loads,
            m2: m2_loads,
            nodes: self.nodes,
            n_fm: n,
            step: 0,
            sampling_frequency: 20,
            upsampling_frequency: self.upsampling_frequency,
        })
    }
}
impl Builder<ZOH> {
    /// Returns a [CfdLoads] [Builder]
    pub fn zoh<C: Into<String>>(cfd_case: C) -> Self {
        Self {
            cfd_case: cfd_case.into(),
            upsampling_frequency: ZOH(20),
            ..Default::default()
        }
    }
}
impl Builder<FOH> {
    /// Returns a [CfdLoads] [Builder]
    pub fn foh<C: Into<String>>(cfd_case: C, upsampling_frequency: usize) -> Self {
        Self {
            cfd_case: cfd_case.into(),
            upsampling_frequency: FOH(upsampling_frequency),
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct ZOH(usize);
#[derive(Default, Debug)]
pub struct FOH(usize);
/// The CFD loads
#[derive(Default, Debug)]
pub struct CfdLoads<S> {
    oss: Option<Vec<f64>>,
    m1: Option<Vec<f64>>,
    m2: Option<Vec<f64>>,
    nodes: Option<Vec<(String, CS)>>,
    n_fm: usize,
    step: usize,
    sampling_frequency: usize,
    upsampling_frequency: S,
}
impl CfdLoads<ZOH> {
    /// Creates a new [CfdLoads] object
    pub fn zoh<C: Into<String>>(cfd_case: C) -> Builder<ZOH> {
        Builder::zoh(cfd_case)
    }
}
impl CfdLoads<FOH> {
    /// Creates a new [CfdLoads] object
    pub fn foh<C: Into<String>>(cfd_case: C, upsampling_frequency: usize) -> Builder<FOH> {
        Builder::foh(cfd_case, upsampling_frequency)
    }
}

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
}
impl<S> fmt::Display for CfdLoads<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(oss) = self.oss_mean() {
            writeln!(f, "CFD loads in OSS:")?;
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

impl<S> Update for CfdLoads<S> {}

enum FemLoads {}
impl Write for CfdLoads<ZOH> {
    fn write(&self) -> Option<Arc<Data<Vec<f64>, FemLoads>>> {
	if let Some(oss) = self.as
    }}

enum M1Loads {}
impl Write for CfdLoads<ZOH> {
    fn write(&self) -> Option<Arc<Data<Vec<f64>, M1Loads>>> {
    }}

enum M2Loads {}
impl Write for CfdLoads<ZOH> {
    fn write(&self) -> Option<Arc<Data<Vec<f64>, M2Loads>>> {
    }}


impl Client for CfdLoads<ZOH> {
    type I = ();
    type O = Vec<f64>;
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        match (self.oss.as_mut(), self.m1.as_mut(), self.m2.as_mut()) {
            (Some(oss), Some(m1), Some(m2)) => {
                if oss.is_empty() || m1.is_empty() || m2.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![
                        oss.drain(..self.n_fm).collect(),
                        m1.drain(..42).collect(),
                        m2.drain(..42).collect(),
                    ])
                }
            }
            (Some(oss), Some(m1), None) => {
                if oss.is_empty() || m1.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![
                        oss.drain(..self.n_fm).collect(),
                        m1.drain(..42).collect(),
                    ])
                }
            }
            (Some(oss), None, Some(m2)) => {
                if oss.is_empty() || m2.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![
                        oss.drain(..self.n_fm).collect(),
                        m2.drain(..42).collect(),
                    ])
                }
            }
            (Some(oss), None, None) => {
                if oss.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![oss.drain(..self.n_fm).collect()])
                }
            }
            (None, Some(m1), Some(m2)) => {
                if m1.is_empty() || m2.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![m1.drain(..42).collect(), m2.drain(..42).collect()])
                }
            }
            (None, Some(m1), None) => {
                if m1.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![m1.drain(..42).collect()])
                }
            }
            (None, None, Some(m2)) => {
                if m2.is_empty() {
                    log::debug!("CFD Loads have dried out!");
                    None
                } else {
                    Some(vec![m2.drain(..42).collect()])
                }
            }
            (None, None, None) => None,
        }
    }
}

impl Client for CfdLoads<FOH> {
    type I = ();
    type O = Vec<f64>;
    fn produce(&mut self) -> Option<Vec<Self::O>> {
        let r = self.upsampling_frequency.0 / self.sampling_frequency;
        let i = self.step / r;
        let u = (self.step - i * r) as f64 / r as f64;
        self.step += 1;
        let foh = |x: &[f64], n: usize| -> Option<Vec<f64>> {
            if let (Some(y0), Some(y1)) = (x.chunks(n).nth(i), x.chunks(n).nth(i + 1)) {
                Some(
                    y0.iter()
                        .zip(y1.iter())
                        .map(|(y0, y1)| (y1 - y0) * u + y0)
                        .collect(),
                )
            } else {
                None
            }
        };
        let release = |x: Vec<Self::O>| {
            if x.is_empty() {
                log::debug!("CFD Loads have dried out!");
                None
            } else {
                Some(x)
            }
        };
        match (self.oss.as_mut(), self.m1.as_mut(), self.m2.as_mut()) {
            (Some(oss), Some(m1), Some(m2)) => release(
                foh(oss, self.n_fm)
                    .into_iter()
                    .chain(foh(m1, 42))
                    .chain(foh(m2, 42))
                    .collect::<Vec<_>>(),
            ),
            (Some(oss), Some(m1), None) => release(
                foh(oss, self.n_fm)
                    .into_iter()
                    .chain(foh(m1, 42))
                    .collect::<Vec<_>>(),
            ),
            (Some(oss), None, Some(m2)) => release(
                foh(oss, self.n_fm)
                    .into_iter()
                    .chain(foh(m2, 42))
                    .collect::<Vec<_>>(),
            ),
            (Some(oss), None, None) => release(foh(oss, self.n_fm).into_iter().collect::<Vec<_>>()),
            (None, Some(m1), Some(m2)) => release(
                foh(m1, 42)
                    .into_iter()
                    .chain(foh(m2, 42))
                    .collect::<Vec<_>>(),
            ),
            (None, Some(m1), None) => release(foh(m1, 42).into_iter().collect::<Vec<_>>()),
            (None, None, Some(m2)) => release(foh(m2, 42).into_iter().collect::<Vec<_>>()),
            (None, None, None) => None,
        }
    }
}
