use glob::{glob, GlobError, PatternError};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Opd {
    pub mean: f64,
    pub values: Vec<f64>,
    pub mask: Vec<bool>,
}

pub struct DomeSeeingData {
    time_stamp: f64,
    file: PathBuf,
}
pub struct DomeSeeing<const N: usize = 1> {
    data: Vec<DomeSeeingData>,
    counter: Box<dyn Iterator<Item = usize>>,
    current_counter: usize,
    next_counter: usize,
    i: usize,
    y1: Opd,
    y2: Opd,
}

impl<const N: usize> DomeSeeing<N> {
    pub fn new<P: AsRef<str> + std::fmt::Display>(path: P, take: Option<usize>) -> Result<Self> {
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
            data.push(DomeSeeingData {
                time_stamp,
                file: entry?,
            });
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
            y1: Default::default(),
            y2: Default::default(),
        })
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn get(&self, idx: usize) -> Result<Opd> {
        let file = File::open(&self.data[idx].file)?;
        Ok(bincode::deserialize_from(&file)?)
    }
}

impl<const N: usize> Iterator for DomeSeeing<N> {
    type Item = Vec<f64>;
    fn next(&mut self) -> Option<Self::Item> {
        let i_cfd = self.i / N;
        if self.i % N == 0 {
            self.current_counter = self.next_counter;
            self.next_counter = self.counter.next().unwrap();
            self.y1 = self
                .get(self.current_counter)
                .expect("failed to load dome seeing data file");
            self.y2 = self
                .get(self.next_counter)
                .expect("failed to load dome seeing data file");
        };
        let alpha = (self.i - i_cfd * N) as f64 / N as f64;
        let y1 = &self.y1;
        let y2 = &self.y2;
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
            DomeSeeing::new("/fsx/CASES/zen30az000_OS7/", Some(n)).unwrap();
        assert_eq!(n, dome_seeing.len());
    }
}
#[test]
fn next() {
    let n = 3;
    const N: usize = 5;
    let mut dome_seeing: DomeSeeing<N> =
        DomeSeeing::new("/fsx/CASES/zen30az000_OS7/", Some(n)).unwrap();
    let mut vals = vec![];
    for i in 0..20 {
        let opd = dome_seeing.next().unwrap();
        vals.push(1e9 * opd[123456]);
        if i % N == 0 {
            println!("{:9.3} *", vals.last().unwrap());
        } else {
            println!("{:9.3}", vals.last().unwrap());
        }
    }
    //assert_eq!(vals[0], *vals.last().unwrap());
}
