use interface::{Update, Read, Data,UID, Write};
use nng::{Error, Protocol, Socket, Message};

pub struct Dcs {
    socket: Socket
} 

impl Dcs {
    pub fn new(url: &str) -> Result<Self,Error> {
        let socket = Socket::new(Protocol::Rep0)?;
        socket.listen(url)?;
        Ok(Self { socket })

    }
}

impl Update for Dcs {
}

#[derive(UID)]
pub enum OcsRequest {}

impl Read<OcsRequest> for Dcs {
    fn read(&mut self, data: Data<OcsRequest>) {
        let mut msg = self.socket.recv().expect("failed to receive");
    }
}

#[derive(UID)]
pub enum DcsReply {}

impl Write<DcsReply> for Dcs{
    fn write(&mut self) -> Option<Data<DcsReply>> {
        let msg = Message::new();
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
