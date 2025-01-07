use std::sync::{Arc, Mutex};
use rand::Rng;
use crate::{atomic_register_client::ClientId, network::Network, quorum::{Quorum, QuorumState}};


#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct NodeId(pub i32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeData {
    data: String,
    version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    ClientWriteRequest((ClientId, String)), 
    ClientReadRequest(ClientId),
    ClientReadResponse(NodeData), 

    CoordinatorWriteRequest((NodeId, NodeData)), 

    CoordinatorReadRequest(NodeId), 
    CoordinatorReadResponse(NodeData), 

    WriteAck(NodeId),       
}

#[derive(Clone, PartialEq, Eq)]
enum NodeState {
    Working,
    Restrtart, 
    Breaking, 
}

#[derive(Clone)]
pub struct Node {
    id: NodeId,
    node_state: NodeState,
    data: Arc<Mutex<NodeData>>,
    quorum: Arc<Mutex<Quorum>>,
    network: Arc<Network>,
    messages: Arc<Mutex<Vec<Message>>>,
}

impl Node {
    pub fn new(
        id: NodeId,
        quorum: usize,
        network: Arc<Network>,
    ) -> Node {
        Node { 
            id, 
            data: Arc::new(Mutex::new(NodeData { data: "".to_string(), version: 0 })), 
            node_state: NodeState::Working,
            quorum: Arc::new(Mutex::new(Quorum::new(quorum))),
            network,
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run(&mut self) {
        self.start_listen();

        loop {
           // self.generate_random_state();

            if self.node_state == NodeState::Breaking {
                continue;
            } else if self.node_state == NodeState::Restrtart {
                self.node_state = NodeState::Working;
                continue;
            } 
            let message = if let Some(message) = self.messages.lock().unwrap().first().cloned() {
                message
            } else {
                continue
            };

            let node = self.clone();

                if node.id.0 == 0 {
                    println!("Node {:?} got messages: {:?}", node.id, node.messages.lock().unwrap());
                }
            match message.clone() {
                Message::ClientWriteRequest((client_id, data)) => {
                    node.remove_first_message();

                    std::thread::spawn(move || {
                        node.handle_client_write_request(&client_id, data);
                    });
                },
                Message::ClientReadRequest(client_id) => {
                    node.remove_first_message();

                    std::thread::spawn(move || {
                        node.handle_client_read_request(client_id);
                    });
                }, 
                Message::CoordinatorWriteRequest((node_id, data)) => {
                    node.remove_first_message();

                    std::thread::spawn(move || {
                        node.handle_coordinator_write_request(node_id, data);
                    });
                },
                Message::CoordinatorReadRequest(node_id) => {
                    node.remove_first_message();

                    std::thread::spawn(move || {
                        node.handle_coordinator_read_request(node_id);
                    });
                },
                Message::CoordinatorReadResponse(_) => {
                    if !node.quorum.lock().unwrap().is_read_coordinator() {
                        node.remove_first_message();
                    }
                },
                Message::WriteAck(_) => {
                    if !node.quorum.lock().unwrap().is_write_coordinator() {
                        node.remove_first_message();
                    }
                },
                _ => { }
            }
        }
    }

    fn start_listen(&self) {
        let node_id = self.id.clone();
        let messages = Arc::clone(&self.messages);
        let network = Arc::clone(&self.network);
        let quorum = Arc::clone(&self.quorum);

        std::thread::spawn(move || {
            while let Some(message) = network.get_node_msg(&node_id) {
                match message {
                    Message::ClientWriteRequest(_) |  
                    Message::ClientReadRequest(_) |
                    Message::CoordinatorWriteRequest(_) |
                    Message::CoordinatorReadRequest(_) => {
                        messages.lock().unwrap().push(message);
                    },
                    Message::CoordinatorReadResponse(_) => {
                        if quorum.lock().unwrap().is_read_coordinator() {
                            messages.lock().unwrap().push(message);
                        }
                    },
                    Message::WriteAck(_) => {
                        if quorum.lock().unwrap().is_write_coordinator() {
                            messages.lock().unwrap().push(message);
                        }
                    },
                    _ => { }
                }
            }
        });
    }

    fn remove_first_message(&self) {
        if self.messages.lock().unwrap().is_empty() {
            return;
        }
        self.messages.lock().unwrap().remove(0);
    }

    fn get_first_msg(&self) -> Option<Message> {
        self.messages.lock().unwrap().first().cloned()
    }

    fn generate_random_state(&mut self) {
        let mut rng = rand::thread_rng();
        let rnd_number = rng.gen_range(0..=100000000);

        if rnd_number <= 5 {
            println!("Node {:?} is breaking.", self.id);
            self.node_state = NodeState::Breaking;
        } else if rnd_number <= 20 {
            println!("Node {:?} is restarting.", self.id);
            self.node_state = NodeState::Restrtart;
        }  
    }

    fn handle_client_write_request(&self, client_id: &ClientId, data: String) {
        println!("Coordinator {:?} got write request from client {:?}", self.id, client_id);

        // Phase 1
        let max_version = self.get_new_data_version();

        // Phase 2
        self.send_write_request(Message::CoordinatorWriteRequest((self.id.clone(), NodeData { data, version: max_version })));

        self.network.send_to_client(client_id, Message::WriteAck(self.id.clone()));
    }

    fn get_new_data_version(&self) -> u32 {
        self.network.send_to_nodes(Message::CoordinatorReadRequest(self.id.clone()), &self.id);

        let mut max_version = self.data.lock().unwrap().version;
        self.quorum.lock().unwrap().quorum_state = QuorumState::WaitingForReadResponse(1);

        while !self.quorum.lock().unwrap().done_read_quorum() {
            let message = if let Some(message) = self.get_first_msg() {
                message.clone()
            } else {
                continue;
            };

            match message {
                Message::CoordinatorReadResponse(data) => {
                    self.remove_first_message();
                    if data.version > max_version {
                        max_version = data.version;
                    }

                    self.quorum.lock().unwrap().increase_read_ack_count();
                },
                _ => {}
            }
        }

        max_version += 1;

        self.quorum.lock().unwrap().go_to_waiting_requst();

        max_version
    }

    fn send_write_request(&self, message: Message) {
        self.network.send_to_nodes(message, &self.id);

        self.quorum.lock().unwrap().quorum_state = QuorumState::WaitingForWriteAck(1);

        println!("Phase 2");

        while !self.quorum.lock().unwrap().done_write_quorum() {
            let message = if let Some(message) = self.get_first_msg() {
                message.clone()
            } else {
                continue;
            };

            println!("message2: {:?}", message);
            println!("messages2: {:?}", self.messages.lock().unwrap());

            match message {
                Message::WriteAck(node_id) => {
                    self.remove_first_message();
                    if self.quorum.lock().unwrap().node_ids.contains(&node_id) {
                        continue;
                    }
                    self.quorum.lock().unwrap().node_ids.push(node_id);
                    self.quorum.lock().unwrap().increase_write_ack_count();
                },
                _ => {
                    self.messages.lock().unwrap().retain(|msg| {
                        match msg {
                            Message::CoordinatorReadResponse(_) => false,
                            _ => true,
                        }
                    });    
                }
            }
        }

        self.quorum.lock().unwrap().go_to_waiting_requst();
    }
    
    fn handle_client_read_request(&self, client_id: ClientId) {
        println!("Coordinator {:?} got read request from client {:?}", self.id, client_id);

        // Phase 1
        self.network.send_to_nodes(Message::CoordinatorReadRequest(self.id.clone()), &self.id);
        self.quorum.lock().unwrap().quorum_state = QuorumState::WaitingForReadResponse(1);

        while !self.quorum.lock().unwrap().done_read_quorum() {
            let message = if let Some(message) = self.get_first_msg() {
                message.clone()
            } else {
                continue;
            };
            println!("messages: {:?}", self.messages.lock().unwrap());
            match message {
                Message::CoordinatorReadResponse(data) => {
                    self.remove_first_message();
                    self.quorum.lock().unwrap().node_datas.push(data);
                    self.quorum.lock().unwrap().increase_read_ack_count();
                },
                _ => {}
            }
        }

        let mut newest_data = self.data.lock().unwrap().clone();
        let mut nodes_havent_newest_data = false;
        let data_version = self.data.lock().unwrap().version;

        for data in self.quorum.lock().unwrap().node_datas.iter() {
            if data.version > data_version {
                newest_data = data.clone();
            } else if data.version != data_version {
                nodes_havent_newest_data = true;
            }
        }

        self.quorum.lock().unwrap().go_to_waiting_requst();

        // Phase 2
        if nodes_havent_newest_data {
            self.quorum.lock().unwrap().quorum_state = QuorumState::WaitingForWriteAck(1);

            self.network.send_to_nodes(Message::CoordinatorWriteRequest((self.id.clone(), newest_data.clone())), &self.id);

            while !self.quorum.lock().unwrap().done_write_quorum() {
                let message = if let Some(message) = self.get_first_msg() {
                    message.clone()
                } else {
                    continue;
                };

                match message {
                    Message::WriteAck(_) => {
                        self.remove_first_message();
                        self.quorum.lock().unwrap().increase_write_ack_count();
                    },
                    _ => {}
                }
            }

            self.quorum.lock().unwrap().go_to_waiting_requst();
        }

        self.network.send_to_client(&client_id, Message::ClientReadResponse(newest_data));
    }
    
    fn handle_coordinator_read_request(&self, node_id: NodeId) {
        println!("Node {:?} got read request from coordinator", self.id);

        self.network.send_to_node(&node_id, Message::CoordinatorReadResponse(self.data.lock().unwrap().clone()));
    }
    
    fn handle_coordinator_write_request(&self, node_id: NodeId, new_data: NodeData) {
        println!("Node {:?} got write request from coordinator with data: {:?}", self.id, new_data);

        *self.data.lock().unwrap() = new_data;
        self.network.send_to_node(&node_id, Message::WriteAck(self.id.clone()));
    }
}