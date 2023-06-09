use super::{Control, ModesIntegrator, ScalarIntegrator};
use crate::{ResidualM2modes, ResidualPistonMode};
use gmt_dos_clients::interface::{Data, Read, Update, Write};
use gmt_dos_clients_crseo::M2modes;

/// Control system for the PWFS
pub struct PwfsIntegrator<P: Control, O: Control> {
    n_mode: usize,
    piston_integrator: ModesIntegrator<P>,
    others_integrator: ModesIntegrator<O>,
    // others_integrator: ModesDblIntegrator,
    // hdfs: Vec<HdfsOrPwfs<f64>>,
}

impl PwfsIntegrator<ScalarIntegrator<f64>, ScalarIntegrator<f64>> {
    /// Creates a new PWFS control system with a `gain`
    pub fn single_single(n_mode: usize, gain: f64) -> Self {
        Self {
            n_mode,
            piston_integrator: ModesIntegrator::new(7, gain),
            others_integrator: ModesIntegrator::single((n_mode - 1) * 7, gain),
            // others_integrator: ModesDblIntegrator::new((n_mode - 1) * 7),
            // hdfs: vec![HdfsOrPwfs::Hdfs(Default::default()); 7],
        }
    }
    pub fn new(n_mode: usize, gain: f64) -> Self {
        Self {
            n_mode,
            piston_integrator: ModesIntegrator::new(7, gain),
            others_integrator: ModesIntegrator::single((n_mode - 1) * 7, gain),
            // others_integrator: ModesDblIntegrator::new((n_mode - 1) * 7),
            // hdfs: vec![HdfsOrPwfs::Hdfs(Default::default()); 7],
        }
    }
}

impl<P: Control, O: Control> Update for PwfsIntegrator<P, O> {
    fn update(&mut self) {
        for scint in self.piston_integrator.scint.iter_mut() {
            scint.step();
        }
        for scint in self.others_integrator.scint.iter_mut() {
            scint.step()
        }
    }
}
impl<P: Control, O: Control> Read<ResidualPistonMode> for PwfsIntegrator<P, O> {
    fn read(&mut self, data: Data<ResidualPistonMode>) {
        data.iter()
            .zip(self.piston_integrator.scint.iter_mut())
            .for_each(|(&data, scint)| scint.set_y(scint.get_y() - data));
    }
}

impl<P: Control, O: Control> Read<ResidualM2modes> for PwfsIntegrator<P, O> {
    fn read(&mut self, data: Data<ResidualM2modes>) {
        data.iter()
            .step_by(self.n_mode)
            .zip(self.piston_integrator.scint.iter_mut())
            .for_each(|(&data, scint)| scint.set_u(data));
        let mut scint_iter_mut = self.others_integrator.scint.iter_mut();
        data.chunks(self.n_mode).for_each(|data| {
            data.iter().skip(1).for_each(|&data| {
                scint_iter_mut.next().map(|scint| scint.set_u(data));
            })
        });
    }
}


impl<P: Control, O: Control> Write<M2modes> for PwfsIntegrator<P, O> {
    fn write(&mut self) -> Option<Data<M2modes>> {
        let mut others_scint_iter = self.others_integrator.scint.iter();
        let data: Vec<_> = self
            .piston_integrator
            .scint
            .iter()
            .flat_map(|scint| {
                let mut modes = vec![scint.get_y()];
                for _ in 0..(self.n_mode - 1) {
                    others_scint_iter
                        .next()
                        .map(|scint| modes.push(scint.get_y()));
                }
                modes
            })
            .collect();
        Some(data.into())
    }
}
