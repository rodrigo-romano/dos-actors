use std::io;

use interface::{UniqueIdentifier, Update, Write};

use crate::{Connector, DcsData, Pull};

use super::{Dcs, DcsIO};

impl<S, D, const B: usize> Update for Dcs<Pull, S, D, B>
where
    S: Connector<Pull> + io::Read + Send + Sync,
    D: Default + DcsData + Send + Sync,
{
    fn update(&mut self) {
        log::debug!("DCS pull update");
        match self.socket.read(&mut self.buffer) {
            Ok(count) if count > 0 => {
                if let Err(e) = self.data.decode(self.buffer.as_mut_slice()) {
                    panic!("DCS error: {:?}", e);
                }
            }
            Ok(_) => panic!("DCS socket read 0 bytes"),
            Err(e) => panic!("DCS error: {:?}", e),
        }
    }
}

impl<U: DcsIO + UniqueIdentifier, S, D, const B: usize> Write<U> for Dcs<Pull, S, D, B>
where
    S: Connector<Pull> + io::Read + Send + Sync,
    D: Default + DcsData + Send + Sync + Write<U>,
{
    fn write(&mut self) -> Option<interface::Data<U>> {
        <D as Write<U>>::write(&mut self.data)
    }
}
