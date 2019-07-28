
use crate::simulated_storage::SimulatedStorage;
// First time I've used many of these apis. To be honest I found this one a
// particularly neat api, but I also don't really know what I'm doing
use std::sync::mpsc::{Receiver, Sender};
use std::collections::HashMap;

use raft::RawNode;
use raft::eraftpb::Message;
use raft::Config;
use raft::Peer;

pub struct Node {
    raft_group: Option<RawNode<SimulatedStorage>>,
    mailbox: Receiver<Message>,
    mailboxes: HashMap<u64, Sender<Message>>,
}

impl Node {
    pub fn create_node(
        is_leader: bool,
        id: u64,
        mailbox: Receiver<Message>,
        mailboxes: HashMap<u64, Sender<Message>>
    ) -> Self {

        // From my point of view there doesn't seem to be a reason to modify the defaults
        let cfg = Config {
            // The unique ID for the Raft node.
            id,
            // needed for logging capabilities
            tag: format!("[{}]", id),
            ..Default::default()
        };

        // Create storage for our raft node. I still do not know what I'm doing
        let storage = SimulatedStorage::new();

        // get peers from the map which contains all the nodes in the network.
        // exclude yourself from this
        let peers_ids = mailboxes
            .keys()
            .cloned()
            .collect::<Vec<u64>>();

        let mut peers = Vec::with_capacity(peers_ids.len() - 1);

        for id in peers_ids {
            let peer = Peer {
                id,
                ..Default::default()
            };
            peers.push(peer);
        }

        // the leader is part of its own group, others are not
        if is_leader {
            Node {
                raft_group: Option::Some(RawNode::new(&cfg, storage).unwrap()),
                mailbox,
                mailboxes,
            }
        } else {
            Node {
                raft_group: Option::None,
                mailbox,
                mailboxes,
            }
        }
    }
}




