use std::{collections::HashMap, sync::{mpsc, Arc, Mutex}};

use crate::{atomic_register_client::ClientId, node::{Message, NodeId}};

// Network simulation

pub struct Network {
    node_senders: HashMap<NodeId, mpsc::Sender<Message>>,
    node_receivers: HashMap<NodeId, Arc<Mutex<mpsc::Receiver<Message>>>>,

    client_senders: HashMap<ClientId, mpsc::Sender<Message>>,
    client_receivers: HashMap<ClientId, Arc<Mutex<mpsc::Receiver<Message>>>>,

    coordinator_id: NodeId,
}   

impl Network {
    pub fn new(
        node_senders: HashMap<NodeId, mpsc::Sender<Message>>,
        node_receivers: HashMap<NodeId, Arc<Mutex<mpsc::Receiver<Message>>>>,

        client_senders: HashMap<ClientId, mpsc::Sender<Message>>,
        client_receivers: HashMap<ClientId, Arc<Mutex<mpsc::Receiver<Message>>>>,
    ) -> Network {
        Network {
            node_senders,
            node_receivers,
            client_senders,
            client_receivers,
            coordinator_id: NodeId(0),
        }
    }

    pub fn get(&self, client_id: &ClientId) -> Option<Message> {
        self.client_receivers.get(client_id).unwrap().lock().unwrap().recv().ok()
    }

    pub fn send(&self, message: Message) {
        self.send_to_node(&self.coordinator_id, message);
    }

    pub fn send_to_node(&self, node_id: &NodeId, message: Message) {
        self.node_senders.get(node_id).unwrap().send(message.clone()).unwrap();
    }

    pub fn send_to_nodes(&self, message: Message, node_id: &NodeId) {
        for (current_node_id, sender) in &self.node_senders {
            if *current_node_id != *node_id {
                sender.send(message.clone()).unwrap();
            }
        }
    }

    pub fn get_node_msg(&self, node_id: &NodeId) -> Option<Message> {
        self.node_receivers.get(node_id).unwrap().lock().unwrap().recv().ok()
    }

    pub fn send_to_client(&self, client_id: &ClientId, message: Message) {
        self.client_senders.get(client_id).unwrap().send(message.clone()).unwrap();
    }
}