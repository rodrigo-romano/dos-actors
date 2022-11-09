use super::DeltaOPD;
use crate::{Opds, Result};
use parse_monitors::cfd;
use rand::{thread_rng, Rng};
use std::{
    f64::INFINITY,
    ops::{Add, Deref, DerefMut},
    rc::Rc,
    time::Instant,
};

pub struct DeltaOPDSet(Vec<DeltaOPD>);
impl Deref for DeltaOPDSet {
    type Target = Vec<DeltaOPD>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DeltaOPDSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl DeltaOPDSet {
    pub fn builder() -> DeltaOPDSetBuilder {
        Default::default()
    }
}
impl Add for DeltaOPDSet {
    type Output = DeltaOPDSet;

    fn add(self, rhs: Self) -> Self::Output {
        let DeltaOPDSet(mut a) = self;
        let DeltaOPDSet(mut b) = rhs;
        a.append(&mut b);
        Self(a)
    }
}

pub struct DeltaOPDSetBuilder {
    n_baseline: usize,
    min_baseline: f64,
    max_baseline: f64,
    cfd_case: cfd::CfdCase<2021>,
    opds: Option<Rc<Opds>>,
}
impl Default for DeltaOPDSetBuilder {
    fn default() -> Self {
        Self {
            n_baseline: 100_000,
            min_baseline: 0f64,
            max_baseline: INFINITY,
            cfd_case: cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7).unwrap(),
            opds: None,
        }
    }
}
impl DeltaOPDSetBuilder {
    pub fn n_baseline(mut self, n_baseline: usize) -> Self {
        self.n_baseline = n_baseline;
        self
    }
    pub fn min_baseline(mut self, min_baseline: f64) -> Self {
        self.min_baseline = min_baseline;
        self
    }
    pub fn max_baseline(mut self, max_baseline: f64) -> Self {
        self.max_baseline = max_baseline;
        self
    }
    pub fn opds(mut self, opds: Rc<Opds>) -> Self {
        self.opds = Some(opds);
        self
    }
    pub fn cfd_case(
        mut self,
        zenith_angle: u32,
        azimuth: u32,
        enclosure: &str,
        wind_speed: u32,
    ) -> Result<Self> {
        self.cfd_case =
            cfd::CfdCase::<2021>::colloquial(zenith_angle, azimuth, enclosure, wind_speed)?;
        Ok(self)
    }
    pub fn cfd_case_id(mut self, id: usize) -> Result<Self> {
        self.cfd_case = cfd::Baseline::<2021>::default()
            .into_iter()
            .nth(id)
            .ok_or(cfd::CfdError::DataFile(format!("CFD CASE#{id}")))?;
        Ok(self)
    }
    pub fn build(self) -> Result<DeltaOPDSet> {
        let opds = self.opds.unwrap_or_else(|| {
            let now = Instant::now();
            let path = cfd::Baseline::<2021>::path().join(self.cfd_case.to_string());
            println!("Collecting OPDs samples from {path:?} ...");
            let opds: Opds = Opds::new(path).expect("failed to load Opds");
            println!(" ... in {}ms", now.elapsed().as_millis());

            Rc::new(opds)
        });
        let n_xy = 104;
        let delta = 0.25;

        let n = opds.mask.iter().filter(|&&m| m).count();
        let n_sample = opds.values.len() / n;

        let n_baseline = self.n_baseline * n_sample;
        println!(
            "Computing {:} OPD finite differences ]{:.2},{:.2}]m ...",
            n_baseline, self.min_baseline, self.max_baseline
        );
        let now = Instant::now();

        let n_xy2 = n_xy * n_xy;
        let mut xy: Vec<(f64, f64)> = Vec::with_capacity(n_xy2);
        let mut mask_iter = opds.mask.iter();
        for i in 0..n_xy {
            for j in 0..n_xy {
                if *mask_iter.next().unwrap() {
                    let x = (i as f64) * delta;
                    let y = (j as f64) * delta;
                    xy.push((x, y));
                }
            }
        }
        let mut rng = thread_rng();
        let mut delta_opd_set: Vec<DeltaOPD> = Vec::with_capacity(n_baseline);
        while delta_opd_set.len() < n_baseline {
            let i = rng.gen_range(0..xy.len());
            let j = rng.gen_range(0..xy.len());
            let (x1, y1) = xy[i];
            let (x2, y2) = xy[j];
            let x = x2 - x1;
            let y = y2 - y1;
            let r = x.hypot(y);
            if r <= self.min_baseline || r > self.max_baseline {
                continue;
            }
            let o = y.atan2(x);
            delta_opd_set.extend(opds.values.chunks(n).enumerate().map(|(t, opd)| {
                let opd1 = opd[i];
                let opd2 = opd[j];
                let delta_opd = opd2 - opd1;
                DeltaOPD { r, o, t, delta_opd }
            }));
        }
        println!(" ... in {}ms", now.elapsed().as_millis());

        Ok(DeltaOPDSet(delta_opd_set))
    }
}
