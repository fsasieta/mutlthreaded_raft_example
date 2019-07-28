
use std::sync::{Arc, RwLock};

use protobuf::Message;

use raft::prelude::*;

use raft::{self, Config, Error, Storage, StorageError};
use raft::util::limit_size;

/// Actual storage container. Only to be accessed by the SimulatedStorageRaft
///
/// I have to admit that much of this design is not novel,
/// however I liked the separation of concerns between the data structure and
/// the atomicity of the operations, hence I chose to keep a good amount of the implementation
/// writing it out helps me understand the api though
///
/// Additionally I relied heavily in the RustConf talk located at https://www.youtube.com/watch?v=MSrcdhGRsOE
/// since I had no experience with this algorithm nor exposure to this library before whatsoever
struct SimulatedStorageRaft {
    state: HardState,
    raft_entries: Vec<Entry>,
    snapshot: Snapshot
}

impl SimulatedStorageRaft {

    fn entries(&self, low: u64, high: u64, max_size: u64) -> Result<Vec<Entry>, Error> {
        Ok(self.raft_entries.clone())
    }

    fn term(&self, idx: u64) -> Result<u64, Error> {
        //The first possible index is always 1
        let dummy_index = self.first_index() - 1;
        if idx < dummy_index || idx > self.last_index() {
            return Ok(0u64);
        }


        Err(Error::ViolatesContract("did not finish implementing this!".to_string()))
    }

    fn first_index(&self) -> u64 {
        match self.raft_entries.first() {
            Some(elem) => elem.index,
            None => self.snapshot.get_metadata().index + 1,
        }
    }

    fn last_index(&self) -> u64 {
        match self.raft_entries.last() {
            Option::Some(value) => value.index,
            Option::None => 1, // I will be unhappy if I don't delete this
        }
    }
}

impl Default for SimulatedStorageRaft {
    fn default() -> SimulatedStorageRaft {
        SimulatedStorageRaft {
            state: Default::default(),
            raft_entries: Vec::new(),
            snapshot: Default::default(),
        }
    }
}

/// Struct encapsulates the atomic operations needed
/// Definitely not a novel data structure design on my part, but the pattern is pretty helpful
#[derive(Clone, Default)]
pub struct SimulatedStorage {
    storage: Arc<RwLock<SimulatedStorageRaft>>,
}

impl SimulatedStorage {
    pub fn new() -> SimulatedStorage {
        SimulatedStorage {
            ..Default::default()
        }
    }

    /// I use a configuration on the node "storage", but don't put any data in it
    pub fn new_with_conf(config: Config) -> SimulatedStorage {
        let storage = SimulatedStorage::new();
        assert!(!storage.initial_state().unwrap().hard_state.is_initialized());
        // initialize the storage with nothing in it
        storage.storage.write().unwrap().snapshot.set_data(vec![]);

        storage
    }
}

/// Implementation of the in-memory storage struct required by the raft library
/// Again, I continue to not know what I'm doing
///
impl Storage for SimulatedStorage {

    fn initial_state(&self) -> Result<RaftState, Error> {
        let read_lock = self.storage.read().unwrap();

        // When we initialize the storage of a node,
        // we need to check whether there is an election
       if read_lock.snapshot.get_metadata().has_pending_membership_change() {
           let mut result: raft::RaftState = raft::RaftState ::new (
               read_lock.state.clone(),
               read_lock.snapshot.get_metadata().get_conf_state().clone()
           );

           // need a way to modify the "private values"
           return Ok(result)
       } else {
           return Ok(RaftState ::new(
               read_lock.state.clone(),
               read_lock.snapshot.get_metadata().get_conf_state().clone()
           ))
       }
    }

    // It was hard to be innovative here as the checks that needed to be made were somewhat standard
    fn entries(&self, low: u64, high: u64, max_size: impl Into<Option<u64>>) -> Result<Vec<Entry>, Error> {
        let max_size = max_size.into();
        let read_lock = self.storage.read().unwrap();
        if low < read_lock.first_index() {
            return Err(Error::Store(StorageError::Compacted))
        } else if high > read_lock.last_index() + 1 {
            panic!("Index out of bounds: last index: {}, high {}", self.storage.read().unwrap().last_index() + 1, high);
        } else {
            let (lower_bound, upper_bound) = ((low - read_lock.raft_entries[0].index),
                                              (high - read_lock.raft_entries[0].index));
            let mut slice_of_entries = read_lock.raft_entries[lower_bound as usize..upper_bound as usize].to_vec();
            limit_size(&mut slice_of_entries, max_size);
            Ok(slice_of_entries)
        }
    }

    /// Returns the term of entry idx, which must be in the range
    /// [first_index()-1, last_index()]. The term of the entry before
    /// first_index is retained for matching purpose even though the
    /// rest of that entry may not be available.
    fn term(&self, idx: u64) -> Result<u64, Error> {
        let read_lock = self.storage.read().unwrap();
        read_lock.term(idx)
    }

    fn first_index(&self) -> Result<u64, Error> {
        Ok(self.storage.read().unwrap().first_index())
    }

    fn last_index(&self) -> Result<u64, Error> {
        Ok(self.storage.read().unwrap().last_index())
    }

    fn snapshot(&self, request_index: u64) -> Result<Snapshot, Error> {
        if self.storage.read().unwrap().snapshot.get_metadata().index < request_index {
            panic!("The requested index is less that the snapshot index")
        } else {
            Ok(self.storage.read().unwrap().snapshot.clone())
        }
    }
}

