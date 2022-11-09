mod delta_opd_set;
pub use delta_opd_set::{DeltaOPDSet, DeltaOPDSetBuilder};
mod delta_opd_subset;
pub use delta_opd_subset::DeltaOPDSubset;
mod structure_function;
pub use structure_function::{
    StructureFunction, StructureFunctionSample, StructureFunctionSubSample,
};

#[derive(Debug)]
pub struct DeltaOPD {
    // polar coordinates radius [m]
    pub r: f64,
    // polar coordinates azimuth [rd]
    pub o: f64,
    // time
    pub t: usize,
    // OPD difference [m]
    pub delta_opd: f64,
}

impl<'a> FromIterator<&'a DeltaOPD> for DeltaOPDSubset<'a> {
    fn from_iter<T: IntoIterator<Item = &'a DeltaOPD>>(iter: T) -> Self {
        DeltaOPDSubset(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a StructureFunction> for StructureFunctionSubSample<'a> {
    fn from_iter<T: IntoIterator<Item = &'a StructureFunction>>(iter: T) -> Self {
        StructureFunctionSubSample {
            sf: iter.into_iter().collect(),
        }
    }
}

pub enum DeltaOpdParam {
    Baseline(f64),
    Time(usize),
}
impl PartialOrd<DeltaOpdParam> for DeltaOPD {
    fn partial_cmp(&self, other: &DeltaOpdParam) -> Option<std::cmp::Ordering> {
        match other {
            DeltaOpdParam::Baseline(b) => self.r.partial_cmp(b),
            DeltaOpdParam::Time(t) => self.t.partial_cmp(t),
        }
    }
}
impl PartialEq<DeltaOpdParam> for DeltaOPD {
    fn eq(&self, other: &DeltaOpdParam) -> bool {
        match *other {
            DeltaOpdParam::Baseline(b) => self.r == b,
            DeltaOpdParam::Time(t) => self.t == t,
        }
    }
}
impl PartialOrd<DeltaOpdParam> for StructureFunction {
    fn partial_cmp(&self, other: &DeltaOpdParam) -> Option<std::cmp::Ordering> {
        match other {
            DeltaOpdParam::Baseline(b) => self.baseline.partial_cmp(b),
            _ => None,
        }
    }
}
impl PartialEq<DeltaOpdParam> for StructureFunction {
    fn eq(&self, other: &DeltaOpdParam) -> bool {
        match *other {
            DeltaOpdParam::Baseline(b) => self.baseline == b,
            _ => false,
        }
    }
}
impl From<DeltaOPDSet> for StructureFunctionSample {
    fn from(mut dopds: DeltaOPDSet) -> Self {
        dopds.sort_by(|a, b| a.r.partial_cmp(&b.r).unwrap());
        let mut nn = 0;
        let mut sf = vec![];
        while let Some(r0) = dopds.get(nn).as_ref().map(|delta_opd| delta_opd.r) {
            let (sum, squared_sum, n) = dopds
                .iter()
                .skip(nn)
                .filter(|delta_opd| delta_opd.r == r0)
                .enumerate()
                .fold(
                    (0f64, 0f64, 0usize),
                    |(mut sum, mut squared_sum, _), (i, delta_opd)| {
                        let d_opd = delta_opd.delta_opd;
                        sum += d_opd;
                        squared_sum += d_opd * d_opd;
                        (sum, squared_sum, i + 1)
                    },
                );
            nn += n;
            let var = squared_sum / n as f64 - (sum / n as f64).powi(2);
            sf.push(StructureFunction::new(r0, var))
        }
        StructureFunctionSample::new(sf)
    }
}
impl<'a> From<DeltaOPDSubset<'a>> for StructureFunctionSample {
    fn from(mut dopds: DeltaOPDSubset<'a>) -> Self {
        dopds.sort_by(|a, b| a.r.partial_cmp(&b.r).unwrap());
        let mut nn = 0;
        let mut sf = vec![];
        while let Some(r0) = dopds.get(nn).as_ref().map(|delta_opd| delta_opd.r) {
            let (sum, squared_sum, n) = dopds
                .iter()
                .skip(nn)
                .filter(|delta_opd| delta_opd.r == r0)
                .enumerate()
                .fold(
                    (0f64, 0f64, 0usize),
                    |(mut sum, mut squared_sum, _), (i, delta_opd)| {
                        let d_opd = delta_opd.delta_opd;
                        sum += d_opd;
                        squared_sum += d_opd * d_opd;
                        (sum, squared_sum, i + 1)
                    },
                );
            nn += n;
            let var = squared_sum / n as f64 - (sum / n as f64).powi(2);
            sf.push(StructureFunction::new(r0, var))
        }
        StructureFunctionSample::new(sf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_set() {
        let dopds = DeltaOPDSet::builder().build().unwrap();
        assert_eq!(dopds.len(), 2001 * 100_000);
    }

    #[test]
    fn subset() {
        let dopds = DeltaOPDSet::builder().build().unwrap();
        let dopds_t: DeltaOPDSubset = dopds
            .iter()
            .filter(|&x| *x == DeltaOpdParam::Time(1))
            .collect();
        assert_eq!(dopds_t.len(), 100_000);
    }

    #[test]
    fn filter() {
        let dopds = DeltaOPDSet::builder().build().unwrap();
        let dopds_t: DeltaOPDSubset = dopds
            .iter()
            .filter(|&x| {
                *x > DeltaOpdParam::Baseline(12f64)
                    && *x <= DeltaOpdParam::Baseline(12.5)
                    && *x == DeltaOpdParam::Time(387)
            })
            .collect();
        println!("{:#?}", &dopds_t[..3]);
        println!("...");
        println!("{:#?}", &dopds_t[dopds_t.len() - 3..]);
    }

    #[test]
    fn structure_function() {
        let dopds = DeltaOPDSet::builder().build().unwrap();
        let dopds_t: DeltaOPDSubset = dopds
            .iter()
            .filter(|&x| *x == DeltaOpdParam::Time(1))
            .collect();
        let sf: StructureFunctionSample = dopds_t.into();
        println!("SF size: {}", sf.len());
    }

    #[test]
    fn power_law() {
        let dopds = DeltaOPDSet::builder().build().unwrap();
        let dopds_inner: DeltaOPDSubset = dopds
            .iter()
            .filter(|&x| *x <= DeltaOpdParam::Baseline(2f64))
            .collect();
        for t in (0..2000).step_by(500) {
            let dopds_inner_t: DeltaOPDSubset = dopds_inner
                .iter()
                .filter(|&&x| *x == DeltaOpdParam::Time(t))
                .map(|x| *x)
                .collect();
            let mut sf: StructureFunctionSample = dopds_inner_t.into();
            let ac = sf.power_law_fit();
            // println!("Subset size: {}", dopds_inner.len());
            // println!("SF size: {}", sf.len());
            println!("Power law: {t:4} {ac:?}");
        }
    }
}
