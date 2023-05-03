use gmt_dos_clients::interface::{Data, Read, Update, Write};
use gmt_dos_clients_io::gmt_m2::asm::segment::{
    FluidDampingForces, AsmCommand, VoiceCoilsForces, VoiceCoilsMotion,
};
use rayon::prelude::*;

use gmt_m2_ctrl_asm_pid_damping::AsmPidDamping;
use gmt_m2_ctrl_asm_preshape_filter::AsmPreshapeFilter;

#[derive(Debug, Default, Clone)]
pub struct AsmSegmentInnerController<const ID: u8> {
    n_mode: usize,
    preshape_filter: Vec<AsmPreshapeFilter>,
    pid_fluid_damping: Vec<AsmPidDamping>,
    km: f64,
    kb: f64,
    ks: Option<Vec<f64>>,
}

impl<const ID: u8> AsmSegmentInnerController<ID> {
    pub fn new(n_mode: usize, ks: Option<Vec<f64>>) -> Self {
        let (preshape_filter, pid_fluid_damping): (Vec<_>, Vec<_>) = (0..n_mode)
            .map(|_| (AsmPreshapeFilter::new(), AsmPidDamping::new()))
            .unzip();
        let km = 0.01120;
        let kb = 33.60;
        Self {
            n_mode,
            preshape_filter,
            pid_fluid_damping,
            km,
            kb,
            ks,
        }
    }
}

impl<const ID: u8> Update for AsmSegmentInnerController<ID> {
    fn update(&mut self) {
        // ASM pre-shape Bessel filter
        let asm_preshape_bessel_filter =
            self.preshape_filter.par_iter_mut().map(|preshape_filter| {
                // preshape_filter.inputs.AO_cmd = *cmd;
                preshape_filter.step();
                (
                    preshape_filter.outputs.cmd_f,
                    // derivatives weighted sum:
                    self.km * preshape_filter.outputs.cmd_f_ddot // km * second_derivative
                    + self.kb * preshape_filter.outputs.cmd_f_dot, // kp * first_derivative
                )
            });

        if let Some(ks) = self.ks.as_ref() {
            let (filtered, filtered_derivatives): (Vec<_>, Vec<_>) =
                asm_preshape_bessel_filter.unzip();
            ks.par_chunks(self.n_mode)
                // dot product between ks row and filtered command: ksf = ks .* filtered_cmd
                .map(|ks_row| {
                    ks_row
                        .iter()
                        .zip(&filtered)
                        .map(|(k, f)| k * f)
                        .sum::<f64>()
                })
                .zip(filtered_derivatives.into_par_iter())
                // add derivatives
                .map(|(ksf, dd)| ksf + dd) // input: asm_FF
                .zip(&filtered) // input: asm_SP
                .zip(&mut self.pid_fluid_damping)
                // .zip(outputs) // ASM PID-fluid-damping outputs
                .for_each(|((asm_ff, asm_sp), pid_fluid_damping)| {
                    // pid_fluid_damping.inputs.asm_FB = *asm_fb;
                    pid_fluid_damping.inputs.asm_SP = *asm_sp;
                    pid_fluid_damping.inputs.asm_FF = asm_ff;
                    pid_fluid_damping.step();
                });
        } else {
            self.pid_fluid_damping
                .par_iter_mut()
                .zip(asm_preshape_bessel_filter) // inputs: asm_FB, asm_SP & asm_FF
                .for_each(|(pid_fluid_damping, (asm_sp, asm_ff))| {
                    // pid_fluid_damping.inputs.asm_FB = *asm_fb;
                    pid_fluid_damping.inputs.asm_SP = asm_sp;
                    pid_fluid_damping.inputs.asm_FF = asm_ff;
                    pid_fluid_damping.step();
                });
        }
    }
}

impl<const ID: u8> Read<AsmCommand<ID>> for AsmSegmentInnerController<ID> {
    fn read(&mut self, data: Data<AsmCommand<ID>>) {
        self.preshape_filter
            .iter_mut()
            .zip(&**data)
            .for_each(|(preshape_filter, data)| preshape_filter.inputs.AO_cmd = *data);
    }
}

impl<const ID: u8> Read<VoiceCoilsMotion<ID>> for AsmSegmentInnerController<ID> {
    fn read(&mut self, data: Data<VoiceCoilsMotion<ID>>) {
        self.pid_fluid_damping
            .iter_mut()
            .zip(&**data)
            .for_each(|(pid_fluid_damping, data)| pid_fluid_damping.inputs.asm_FB = *data);
    }
}

impl<const ID: u8> Write<VoiceCoilsForces<ID>> for AsmSegmentInnerController<ID> {
    fn write(&mut self) -> Option<Data<VoiceCoilsForces<ID>>> {
        let modal_forces = self
            .pid_fluid_damping
            .iter()
            .map(|pid_fluid_damping| pid_fluid_damping.outputs.asm_U);
        Some(Data::new(modal_forces.collect()))
    }
}
impl<const ID: u8> Write<FluidDampingForces<ID>> for AsmSegmentInnerController<ID> {
    fn write(&mut self) -> Option<Data<FluidDampingForces<ID>>> {
        let fluid_damping = self
            .pid_fluid_damping
            .iter()
            .map(|pid_fluid_damping| pid_fluid_damping.outputs.asm_Fd);
        Some(Data::new(fluid_damping.collect()))
    }
}
