use std::{
    fmt::Debug,
    ops::{Mul, Sub, SubAssign},
    sync::Arc,
};

use gmt_dos_clients::interface::{Data, Read, Update, Write, UID};
use gmt_dos_clients_crseo::M2modes;

use crate::PistonMode;

use super::{HdfsOrNot, HdfsOrPwfs, ScalarIntegrator};

struct ModesIntegrator<T> {
    pub scint: ScalarIntegrator<T>,
    pub current: Vec<T>,
    pub residual: Vec<T>,
}
impl<T> ModesIntegrator<T>
where
    T: Default + Copy + Sub<T, Output = T> + Mul<T, Output = T>,
{
    fn new(n_sample: usize, gain: T) -> Self {
        Self {
            scint: ScalarIntegrator::new(gain),
            current: vec![Default::default(); n_sample],
            residual: vec![Default::default(); n_sample],
        }
    }
    /*     pub fn step(&self) -> Vec<T> {
        self.residual
            .iter()
            .zip(&self.current)
            .map(|(&u, &y)| self.scint.step(u, y))
            .collect()
    } */
    fn iter_mut(&mut self) -> impl Iterator<Item = (&mut T, &T)> {
        self.current.iter_mut().zip(&self.residual)
    }
}

/// Control system for the PWFS
pub struct PwfsIntegrator<T> {
    n_mode: usize,
    piston_integrator: ModesIntegrator<T>,
    others_integrator: ModesIntegrator<T>,
    hdfs: Vec<HdfsOrPwfs<T>>,
}
impl<T> PwfsIntegrator<T>
where
    T: Default + Copy + Sub<T, Output = T> + Mul<T, Output = T>,
{
    /// Creates a new PWFS control system with a `gain`
    pub fn new(n_mode: usize, gain: T) -> Self {
        Self {
            n_mode,
            piston_integrator: ModesIntegrator::new(7, gain),
            others_integrator: ModesIntegrator::new((n_mode - 1) * 7, gain),
            hdfs: vec![HdfsOrPwfs::Hdfs(Default::default()); 7],
        }
    }
}

impl<T> Update for PwfsIntegrator<T>
where
    T: Default + Debug + Copy + Sub<T, Output = T> + SubAssign<T> + Mul<T, Output = T>,
{
    fn update(&mut self) {
        let gain = self.piston_integrator.scint.gain;
        for ((y, &u), may_be_pym) in self.piston_integrator.iter_mut().zip(self.hdfs.iter()) {
            match may_be_pym {
                HdfsOrPwfs::Pwfs => {
                    *y -= gain * u;
                }
                HdfsOrPwfs::Hdfs(a1) => *y = *a1,
            }
        }
        let gain = self.others_integrator.scint.gain;
        for (y, &u) in self.others_integrator.iter_mut() {
            *y -= gain * u;
        }
    }
}

#[derive(UID)]
pub enum ResidualM2modes {}

impl Read<ResidualM2modes> for PwfsIntegrator<f64> {
    fn read(&mut self, data: Arc<Data<ResidualM2modes>>) {
        data.iter()
            .step_by(self.n_mode)
            .zip(self.piston_integrator.residual.iter_mut())
            .for_each(|(&data, r)| *r = data);
        data.chunks(self.n_mode)
            .zip(self.others_integrator.residual.chunks_mut(self.n_mode - 1))
            .for_each(|(data, r)| data.iter().skip(1).zip(r).for_each(|(&data, r)| *r = data));
    }
}

impl Read<HdfsOrNot> for PwfsIntegrator<f64> {
    fn read(&mut self, data: Arc<Data<HdfsOrNot>>) {
        self.hdfs = (**data).clone();
    }
}

impl Write<PistonMode> for PwfsIntegrator<f64> {
    fn write(&mut self) -> Option<Arc<Data<PistonMode>>> {
        Some(Arc::new(Data::new(self.piston_integrator.current.clone())))
    }
}

impl Write<M2modes> for PwfsIntegrator<f64> {
    fn write(&mut self) -> Option<Arc<Data<M2modes>>> {
        let data: Vec<_> = self
            .piston_integrator
            .current
            .iter()
            .zip(self.others_integrator.current.chunks(self.n_mode - 1))
            .flat_map(|(p, o)| {
                let mut modes = vec![*p];
                modes.extend_from_slice(o);
                modes
            })
            .collect();
        Some(Arc::new(Data::new(data)))
    }
}
