use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update, Write, UID};
use std::{
    fmt::Debug,
    ops::{Mul, Sub},
    sync::Arc,
};

use crate::PistonMode;

use super::ScalarIntegrator;

/// Selector between HDFS or PWFS piston
#[derive(Debug, Clone)]
pub enum HdfsOrPwfs<T> {
    Pwfs,
    Hdfs(T),
}

/// Control system for the HDFS
pub struct HdfsIntegrator<T> {
    scint: ScalarIntegrator<T>,
    may_be_pym: Vec<HdfsOrPwfs<T>>,
    bound: T,
    current: Vec<T>,
    residual: Vec<T>,
}
impl<T> HdfsIntegrator<T>
where
    T: Default + Copy + Sub<T, Output = T> + Mul<T, Output = T>,
{
    /// Creates a new HDFS control system with a `gain` and the PWFS piston `bound`
    pub fn new(gain: T, bound: T) -> Self {
        Self {
            scint: ScalarIntegrator::new(gain),
            may_be_pym: vec![HdfsOrPwfs::Hdfs(Default::default()); 7],
            bound,
            current: vec![Default::default(); 7],
            residual: vec![Default::default(); 7],
        }
    }
}

impl<T> Update for HdfsIntegrator<T>
where
    T: Default + Debug + Copy + Sub<T, Output = T> + Mul<T, Output = T> + num_traits::float::Float,
{
    fn update(&mut self) {
        self.may_be_pym = self
            .residual
            .iter()
            .zip(&self.current)
            .map(|(&u, &y)| {
                if T::abs(u) > self.bound {
                    HdfsOrPwfs::Hdfs(self.scint.step(u, y))
                } else {
                    HdfsOrPwfs::Pwfs
                }
            })
            .collect();
    }
}

#[derive(UID)]
pub enum ResidualPistonMode {}

impl Read<ResidualPistonMode> for HdfsIntegrator<f64> {
    fn read(&mut self, data: Arc<Data<ResidualPistonMode>>) {
        self.residual = (**data).clone();
    }
}

impl Read<PistonMode> for HdfsIntegrator<f64> {
    fn read(&mut self, data: Arc<Data<PistonMode>>) {
        self.current = (**data).clone();
    }
}

pub enum HdfsOrNot {}
impl UniqueIdentifier for HdfsOrNot {
    type DataType = Vec<HdfsOrPwfs<f64>>;
}
impl Write<HdfsOrNot> for HdfsIntegrator<f64> {
    fn write(&mut self) -> Option<Arc<Data<HdfsOrNot>>> {
        Some(Arc::new(Data::new(self.may_be_pym.clone())))
    }
}
