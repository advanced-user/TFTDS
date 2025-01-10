use crate::node::{NodeData, NodeId};

pub enum QuorumState {
    WaitingForWriteAck(usize), // read acks count
    WaitingForReadResponse(usize), // write acks count
    WaitingForRequest,
}

pub struct Quorum {
    pub acks: usize,
    pub node_datas: Vec<NodeData>,
    pub node_ids: Vec<NodeId>,
    pub quorum_state: QuorumState,
}

impl Quorum {
    pub fn new(acks: usize) -> Quorum {
        Quorum { 
            acks, 
            node_datas: Vec::new(), 
            node_ids: Vec::new(), 
            quorum_state: QuorumState::WaitingForRequest 
        }
    }

    pub fn go_to_waiting_requst(&mut self) {
        self.quorum_state = QuorumState::WaitingForRequest;
        self.node_datas.clear();
        self.node_ids.clear();
    }

    pub fn is_read_coordinator(&self) -> bool {
        match self.quorum_state {
            QuorumState::WaitingForReadResponse(_) => true,
            _ => false,
        }
    }

    pub fn is_waiting_request(&self) -> bool {
        match self.quorum_state {
            QuorumState::WaitingForRequest => true,
            _ => false,
        }
    }

    pub fn is_write_coordinator(&self) -> bool {
        match self.quorum_state {
            QuorumState::WaitingForWriteAck(_) => true,
            _ => false,
        }
    }

    pub fn done_read_quorum(&self) -> bool {
        match self.quorum_state {
            QuorumState::WaitingForReadResponse(count) => count == self.acks,
            _ => false,
        }
    }

    pub fn done_write_quorum(&self) -> bool {
        match self.quorum_state {
            QuorumState::WaitingForWriteAck(count) => count == self.acks,
            _ => false,
        }
    }

    pub fn increase_write_ack_count(&mut self) {
        match self.quorum_state {
            QuorumState::WaitingForWriteAck(ref mut count) => *count += 1,
            _ => panic!("Not in write ack state"),
        }
    }

    pub fn increase_read_ack_count(&mut self) {
        match self.quorum_state {
            QuorumState::WaitingForReadResponse(ref mut count) => *count += 1,
            _ => panic!("Not in read response state"),
        }
    }
}
