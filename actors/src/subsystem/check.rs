use crate::{actor::PlainActor, Check, Task};

use super::{Gateways, Iter, SubSystem, SubSystemIterator};

impl<M, const NI: usize, const NO: usize> Check for SubSystem<M, NI, NO>
where
    M: Gateways + 'static,
    for<'a> SubSystemIterator<'a, M>: Iterator<Item = &'a dyn Check>,
{
    fn check_inputs(&self) -> std::result::Result<(), crate::CheckError> {
        self.gateway_in.check_inputs()?;
        self.gateway_out.check_inputs()?;
        self.system
            .iter()
            .map(|actor| actor.check_inputs())
            .collect::<std::result::Result<Vec<()>, crate::CheckError>>()?;
        Ok(())
    }

    fn check_outputs(&self) -> std::result::Result<(), crate::CheckError> {
        self.gateway_in.check_outputs()?;
        self.gateway_out.check_outputs()?;
        self.system
            .iter()
            .map(|actor| actor.check_outputs())
            .collect::<std::result::Result<Vec<()>, crate::CheckError>>()?;
        Ok(())
    }

    fn n_inputs(&self) -> usize {
        self.gateway_in.n_inputs()
            + self.gateway_out.n_inputs()
            + self
                .system
                .iter()
                .map(|actor| actor.n_inputs())
                .sum::<usize>()
    }

    fn n_outputs(&self) -> usize {
        self.gateway_in.n_outputs()
            + self.gateway_out.n_outputs()
            + self
                .system
                .iter()
                .map(|actor| actor.n_outputs())
                .sum::<usize>()
    }

    fn inputs_hashes(&self) -> Vec<u64> {
        self.gateway_in
            .inputs_hashes()
            .into_iter()
            .chain(self.gateway_out.inputs_hashes().into_iter())
            .chain(self.system.iter().flat_map(|actor| actor.inputs_hashes()))
            .collect()
    }

    fn outputs_hashes(&self) -> Vec<u64> {
        self.gateway_in
            .outputs_hashes()
            .into_iter()
            .chain(self.gateway_out.outputs_hashes().into_iter())
            .chain(self.system.iter().flat_map(|actor| actor.outputs_hashes()))
            .collect()
    }
    fn _as_plain(&self) -> PlainActor {
        let mut subsystem = PlainActor::default();
        subsystem.client = self.get_name();
        let way_in = self.gateway_in.as_plain();
        subsystem.inputs_rate = way_in.inputs_rate;
        subsystem.inputs = way_in.inputs;
        let way_out = self.gateway_out.as_plain();
        subsystem.outputs_rate = way_out.outputs_rate;
        subsystem.outputs = way_out.outputs;
        subsystem
    }
}
