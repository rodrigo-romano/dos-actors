use std::io;

use interface::{Read, UniqueIdentifier, Update};

use crate::{Connector, DcsData, Push};

use super::Dcs;

impl<S, D, const B: usize> Update for Dcs<Push, S, D, B>
where
    S: Connector<Push> + io::Write + Send + Sync,
    D: Default + DcsData + Send + Sync,
{
    fn update(&mut self) {
        log::debug!("DCS push update");
        match self.data.write() {
            Ok(mut buffer) => {
                if let Err(e) = self.socket.write_all(&mut buffer) {
                    panic!("DCS error: {:?}", e)
                }
            }
            Err(e) => panic!("DCS error: {:?}", e),
        }
    }
}

impl<U: UniqueIdentifier, S, D, const B: usize> Read<U> for Dcs<Push, S, D, B>
where
    S: Connector<Push> + io::Write + Send + Sync,
    D: Default + DcsData + Send + Sync + Read<U>,
{
    fn read(&mut self, data: interface::Data<U>) {
        <D as Read<U>>::read(&mut self.data, data);
    }
}
