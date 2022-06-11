use glob::{glob, GlobError, PatternError};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DomeSeeingError {
    #[error("failed to load dome seeing data")]
    Load(#[from] std::io::Error),
    #[error("failed to get dome seeing data path")]
    Glob(#[from] GlobError),
    #[error("failed to find dome seeing file pattern")]
    Pattern(#[from] PatternError),
    #[error("failed to read dome seeing file")]
    Bincode(#[from] bincode::Error),
}

pub type Result<T> = std::result::Result<T, DomeSeeingError>;

//const CFD_SAMPLING_FREQUENCY: f64 = 5f64; // Hz

#[derive(Serialize, Deserialize, Debug)]
pub struct Opd {
    pub mean: f64,
    pub values: Vec<f64>,
    pub mask: Vec<bool>,
}

pub struct DomeSeeingData {
    time_stamp: f64,
    opd: Opd,
}
pub struct DomeSeeing<const N: usize = 1> {
    data: Vec<DomeSeeingData>,
    counter: Box<dyn Iterator<Item = usize>>,
    current_counter: usize,
    next_counter: usize,
    i: usize,
}

impl<const N: usize> DomeSeeing<N> {
    pub fn load<P: AsRef<str> + std::fmt::Display>(path: P, take: Option<usize>) -> Result<Self> {
        let mut data: Vec<DomeSeeingData> = Vec::with_capacity(2005);
        for entry in
            glob(&format!("{}/optvol/optvol_optvol_*", path))?.take(take.unwrap_or(usize::MAX))
        {
            let time_stamp = entry
                .as_ref()
                .ok()
                .and_then(|x| x.file_name())
                .and_then(|x| Path::new(x).file_stem())
                .and_then(|x| x.to_str())
                .and_then(|x| x.split("_").last())
                .and_then(|x| x.parse::<f64>().ok())
                .expect("failed to parse dome seeing time stamp");
            let file = File::open(entry?)?;
            let opd = bincode::deserialize_from(&file)?;
            data.push(DomeSeeingData { time_stamp, opd });
        }
        data.sort_by(|a, b| a.time_stamp.partial_cmp(&b.time_stamp).unwrap());
        let mut counter = Box::new(
            (0..data.len())
                .chain((0..data.len()).skip(1).rev().skip(1))
                .cycle(),
        );
        let next_counter = counter.next().unwrap();
        Ok(Self {
            data,
            counter,
            current_counter: 0,
            next_counter,
            i: 0,
        })
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<const N: usize> Iterator for DomeSeeing<N> {
    type Item = Vec<f64>;
    fn next(&mut self) -> Option<Self::Item> {
        let i_cfd = self.i / N;
        if self.i % N == 0 {
            self.current_counter = self.next_counter;
            self.next_counter = self.counter.next().unwrap();
        };
        let alpha = (self.i - i_cfd * N) as f64 / N as f64;
        let y1 = &self.data[self.current_counter].opd;
        let y2 = &self.data[self.next_counter].opd;
        let opd_i = y1
            .values
            .iter()
            .zip(&y2.values)
            .map(|(y1, y2)| y1 + (y2 - y1) * alpha);
        let mut opd = vec![0f64; y1.mask.len()];
        opd.iter_mut()
            .zip(&y1.mask)
            .filter(|(_, mask)| **mask)
            .zip(opd_i)
            .for_each(|((opd, _), opd_i)| *opd = opd_i);
        self.i += 1;
        Some(opd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load() {
        let n = 3;
        let dome_seeing: DomeSeeing<1> =
            DomeSeeing::load("/fsx/CASES/zen30az000_OS7/", Some(n)).unwrap();
        assert_eq!(n, dome_seeing.len());
    }
}
#[test]
fn next() {
    let n = 3;
    const N: usize = 2;
    let mut dome_seeing: DomeSeeing<N> =
        DomeSeeing::load("/fsx/CASES/zen30az000_OS7/", Some(n)).unwrap();
    let mut vals = vec![];
    for _ in 0..9 {
        let opd = dome_seeing.next().unwrap();
        vals.push(1e6 * opd[123456]);
        println!("{}", vals.last().unwrap());
    }
    assert_eq!(vals[0], *vals.last().unwrap());
}
