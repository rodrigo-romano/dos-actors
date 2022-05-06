use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
    num::ParseFloatError,
    path::Path,
    sync::Arc,
};
use thiserror::Error;

use crate::io::Data;

#[derive(Debug, Error)]
pub enum DTAError {
    #[error("Failed to open DTA file")]
    IO(#[from] std::io::Error),
    #[error("Failed to parse DTA data")]
    Parsing(#[from] ParseFloatError),
}

pub type Result<T> = std::result::Result<T, DTAError>;

#[derive(Debug)]
pub struct Load {
    #[allow(dead_code)]
    time_stamp: f64,
    data: Vec<f64>,
}
#[derive(Debug)]
pub struct CfdLoads(VecDeque<Load>);

impl CfdLoads {
    pub fn new<P: AsRef<Path>>(data_path: P) -> Result<Self> {
        let file = File::open(data_path)?;
        let rdr = BufReader::new(file);
        Ok(Self(
            rdr.lines()
                .skip(12)
                .map(|line| {
                    let a_line = line?;
                    let mut chunks = a_line.split(',');
                    let time_stamp = chunks
                        .next()
                        .map(|x| x.parse::<f64>().map_err(|e| DTAError::Parsing(e)))
                        .unwrap()?;
                    let data = chunks
                        .map(|x| x.parse::<f64>().map_err(|e| DTAError::Parsing(e)))
                        .collect::<Result<Vec<f64>>>()?;
                    Ok(Load { time_stamp, data })
                })
                .collect::<Result<VecDeque<Load>>>()?,
        ))
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Iterator for CfdLoads {
    type Item = Vec<f64>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front().map(|x| x.data)
    }
}

impl crate::Update for CfdLoads {}
#[cfg(feature = "fem")]
impl crate::io::Write<Vec<f64>, fem::fem_io::OSSDTAWind6F> for CfdLoads {
    fn write(
        &mut self,
    ) -> Option<std::sync::Arc<crate::io::Data<Vec<f64>, fem::fem_io::OSSDTAWind6F>>> {
        self.next().map(|x| Arc::new(Data::new(x)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dta_loads() {
        let loads =
            CfdLoads::new("/fsx/DTA/zen30az000_OS7/GMT-DTA-190952_RevB1_WLC00xx.csv").unwrap();
        println!("CFD loads #: {}", loads.len());
        loads.into_iter().take(3).for_each(|l| println!("{l:?}"));
    }
}
