use std::sync::Arc;

use crate::{network::Network, node::Message};

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct ClientId(pub i32);

pub struct AtomicRegisterClinent {
    id: ClientId,
    network: Arc<Network>,
}

impl AtomicRegisterClinent {
    pub fn new(id: ClientId, network: Arc<Network>) -> AtomicRegisterClinent {
        AtomicRegisterClinent {
            id,
            network,
        }
    }

    pub fn write(&self, data: String) {
        self.network.send(Message::ClientWriteRequest((self.id.clone(), data)));

        loop {
            let acks = self.network.get(&self.id);

            match acks {
                Some(Message::WriteAck(node_id)) => {
                    println!("Client {:?} got ack from node: {:?}", self.id, node_id);
                    break;
                },
                _ => {},
            }
        }
    }

    pub fn read(&self) {
        self.network.send(Message::ClientReadRequest(self.id.clone()));

        loop {
            let data = self.network.get(&self.id);

            match data {
                Some(Message::ClientReadResponse(node_data)) => {
                    println!("Client {:?} got data from node: {:?}", self.id, node_data);
                    break;
                },
                _ => {},
            }
        }
    }
}