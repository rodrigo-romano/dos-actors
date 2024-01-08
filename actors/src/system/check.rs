use crate::{
    actor::PlainActor,
    framework::model::{Check, CheckError},
};

use super::System;

impl<T> Check for T
where
    T: System,
{
    fn check_inputs(&self) -> std::result::Result<(), CheckError> {
        self.iter()
            .map(|actor| actor.check_inputs())
            .collect::<std::result::Result<Vec<()>, CheckError>>()?;
        Ok(())
    }

    fn check_outputs(&self) -> std::result::Result<(), CheckError> {
        self.iter()
            .map(|actor| actor.check_outputs())
            .collect::<std::result::Result<Vec<()>, CheckError>>()?;
        Ok(())
    }

    fn n_inputs(&self) -> usize {
        self.iter().map(|actor| actor.n_inputs()).sum::<usize>()
    }

    fn n_outputs(&self) -> usize {
        self.iter().map(|actor| actor.n_outputs()).sum::<usize>()
    }

    fn inputs_hashes(&self) -> Vec<u64> {
        self.iter()
            .flat_map(|actor| actor.inputs_hashes())
            .collect()
    }

    fn outputs_hashes(&self) -> Vec<u64> {
        self.iter()
            .flat_map(|actor| actor.outputs_hashes())
            .collect()
    }
    fn _as_plain(&self) -> PlainActor {
        let mut subsystem = PlainActor::default();
        subsystem.client = self.name().unwrap_or("system".to_string());
        /*         let way_inandout = self.gateway.as_plain();
        subsystem.inputs_rate = way_inandout.inputs_rate;
        subsystem.inputs = way_inandout.inputs;
        subsystem.outputs_rate = way_inandout.outputs_rate;
        subsystem.outputs = way_inandout.outputs; */
        subsystem
    }
}
