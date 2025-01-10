use std::{collections::HashMap, sync::{mpsc, Arc, Mutex}};

use atomic_register::{atomic_register_client::{AtomicRegisterClinent, ClientId}, network::Network, node::{Message, Node, NodeId}};

fn main() {
    let (node_senders, node_receivers) = create_node_network(3);
    let (client_senders, client_receivers) = create_client_network(2);

    let network = Arc::new(Network::new(
        node_senders,
        node_receivers,
        client_senders,
        client_receivers,
    ));

    let client1 = AtomicRegisterClinent::new(ClientId(0), Arc::clone(&network));
    let client2 = AtomicRegisterClinent::new(ClientId(1), Arc::clone(&network));

    let mut nodes = Vec::new();
    for i in 0..3 {
        let node_id = NodeId(i);
        let node = Node::new(node_id.clone(), 2, Arc::clone(&network)); 
        nodes.push(node);
    }

    for mut node in nodes {
        std::thread::spawn(move || {
            node.run();
        });
    }
    
    let handle1 = std::thread::spawn(move || {
        println!("Client1 starts read operation.");
        client1.read();

        println!("Client1 starts write operation.");
        client1.write( "Data 1".to_string());

        println!("Client1 starts read operation.");
        client1.read();
    });
    
    let handle2 = std::thread::spawn(move || {
        println!("Client2 starts read operation.");
        client2.read();

        println!("Client2 starts write operation.");
        client2.write("Data 2".to_string());

        println!("Client2 starts read operation.");
        client2.read();
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

fn create_node_network(node_count: usize) -> (HashMap<NodeId, mpsc::Sender<Message>>, HashMap<NodeId, Arc<Mutex<mpsc::Receiver<Message>>>>) {
    let mut node_senders = HashMap::new();
    let mut node_receivers = HashMap::new();

    for i in 0..node_count {
        let (tx, rx) = mpsc::channel();
        node_senders.insert(NodeId(i as i32), tx);
        node_receivers.insert(NodeId(i as i32), Arc::new(Mutex::new(rx)));
    }

    (node_senders, node_receivers)
}

fn create_client_network(client_count: usize) -> (HashMap<ClientId, mpsc::Sender<Message>>, HashMap<ClientId, Arc<Mutex<mpsc::Receiver<Message>>>>) {
    let mut client_senders = HashMap::new();
    let mut client_receivers = HashMap::new();

    for i in 0..client_count {
        let (tx, rx) = mpsc::channel();
        client_senders.insert(ClientId(i as i32), tx);
        client_receivers.insert(ClientId(i as i32), Arc::new(Mutex::new(rx)));
    }

    (client_senders, client_receivers)
}
