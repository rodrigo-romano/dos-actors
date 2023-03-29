use gmt_dos_clients::interface::{Data, Read, UniqueIdentifier, Update, Write};
use std::sync::Arc;

use crate::{PistonMode, ResidualPistonMode};

use super::{ScalarIntegrator, ScalarIntegratorTrait};

/// Selector between HDFS or PWFS piston
#[derive(Debug, Clone)]
pub enum HdfsOrPwfs<T> {
    Pwfs,
    Hdfs(T),
}

/// Control system for the HDFS
pub struct HdfsIntegrator<T> {
    scint: Vec<ScalarIntegrator<T>>,
    may_be_pym: Vec<HdfsOrPwfs<T>>,
    piston_2_mode: Vec<T>,
    bound: T,
}
impl<T> HdfsIntegrator<T>
where
    T: ScalarIntegratorTrait<T>,
{
    /// Creates a new HDFS control system with a `gain` and the PWFS piston `bound`
    pub fn new(gain: T, piston_2_mode: Vec<T>, bound: T) -> Self {
        Self {
            scint: vec![ScalarIntegrator::new(gain); 7],
            may_be_pym: vec![HdfsOrPwfs::Hdfs(Default::default()); 7],
            piston_2_mode,
            bound,
        }
    }
}

impl<T> Update for HdfsIntegrator<T>
where
    T: ScalarIntegratorTrait<T>,
{
    fn update(&mut self) {
        self.may_be_pym = self
            .scint
            .iter_mut()
            .zip(&self.piston_2_mode)
            .map(|(scint, &a)| {
                scint.step();
                if T::abs(scint.u / a) > self.bound {
                    HdfsOrPwfs::Hdfs(scint.y)
                } else {
                    HdfsOrPwfs::Pwfs
                }
            })
            .collect();
    }
}

impl Read<ResidualPistonMode> for HdfsIntegrator<f64> {
    fn read(&mut self, data: Arc<Data<ResidualPistonMode>>) {
        self.scint
            .iter_mut()
            .zip(data.iter())
            .for_each(|(scint, data)| scint.u = *data);
    }
}

impl Read<PistonMode> for HdfsIntegrator<f64> {
    fn read(&mut self, data: Arc<Data<PistonMode>>) {
        self.scint
            .iter_mut()
            .zip(data.iter())
            .for_each(|(scint, data)| scint.y = *data);
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
