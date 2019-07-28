#[macro_use]
extern crate slog;
extern crate slog_term;

mod node;
mod simulated_storage;

use std::thread;
use std::env::args;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};
use std::convert::TryInto;

use rand::*;

use raft::eraftpb::Message;

use slog::{Logger, Drain};
use slog_term::*;
use std::collections::HashMap;

use node::Node;


/// We default to 1 for the purposes of this exercise
/// We also parse negative integers as positive, but warn the user about it
fn parse_arguments(args: &Vec<String>) -> u64 {
    let mut number_of_nodes = args[1]
        .clone()
        .parse::<u64>()
        .unwrap_or_else(|_unused_arg| {1});
    println!("Starting simulation with {} nodes.\n \
    If you entered something other that a number, you may want stop the program and try again", number_of_nodes);
    number_of_nodes
}

/// Ergonomics function
fn create_x_number_of_mailboxes(num_nodes: u64) -> (Vec<Sender<Message>>, Vec<Receiver<Message>>){
    let (mut sender_vec, mut receiver_vec) = (Vec::new(), Vec::new());
    for _i in 0..num_nodes {
        let (sender, receiver) = mpsc::channel();
        sender_vec.push(sender);
        receiver_vec.push(receiver);
    }

    (sender_vec, receiver_vec)
}

enum Signals {
    Terminate,
}

fn terminate(receiver: &Arc<Mutex<mpsc::Receiver<Signals>>>) -> bool {
    match receiver.lock().unwrap().try_recv() {
        Ok(Signals::Terminate) => true,
        Err(TryRecvError::Empty) => return false,
        Err(TryRecvError::Disconnected) => return true,
    }
}

/// First time initializing a logger in Rust
/// Decided to encapsulate every part of the creation and initialization
fn initialize_and_return_logger() -> slog::Logger {
    // Formats logs
    let decorator = TermDecorator::new().build();
    // Making previous formatter thread safe and able to log to the terminal
    // Also make it panic on errors
    let drain = FullFormat::new(decorator).build().fuse();
    // Previous version was synchronous, make it asynchronous and constrains log size
    let drain = slog_async::Async::new(drain)
        .chan_size(4096)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();
    // Everything before us was configuring where and how our log behaves, this creates
    // the actual logger instance
    let logger = Logger::root(drain, o!());
    logger
}

fn main() {

    let logger = initialize_and_return_logger();

    let number_of_nodes = parse_arguments(&args().collect::<Vec<String>>());
    let (mut sender_mailboxes, mut receiver_mailboxes) = create_x_number_of_mailboxes(number_of_nodes);
    let mut descriptors = Vec::new();

    // Cleanup handlers
    let (sender_stop, recv_stop) = mpsc::channel();
    let recv_stop = Arc::new(Mutex::new(recv_stop));


    // Randomly elect a leader
    let mut rng = rand::thread_rng();
    let random_leader =  rng.gen_range(1, number_of_nodes + 1);

    // 1) Initialize all nodes,
    // 2) Register them with each other,
    // 3) Give each their own thread,
    // 4) Grab all descriptors to them,
    // 5) Move a terminated signal to each thread closure for cleanup
    for (i, recv_mailbox) in receiver_mailboxes.into_iter().enumerate() {

        // Global map of Id -> sender mailbox
        // need to make it in the closure because each node needs an instance of it
        let mailboxes: HashMap<u64, Sender<Message>> = (1..(number_of_nodes + 1))
            .zip(sender_mailboxes.iter().cloned())
            .collect();

        let mut node = if i == random_leader.try_into().unwrap() {
//                    info!("Initializing leader node {}", i);
            Node::create_node(true, i as u64, recv_mailbox, mailboxes)
        } else {
//                    info!("Initializing follower node {}", i);
            Node::create_node(false, i as u64, recv_mailbox, mailboxes)
        };

        // Create a new thread for the node just built
        let recv_stop_clone = recv_stop.clone();
        let descriptor = thread::spawn(move || loop {

            if terminate(&recv_stop_clone) {
                return;
            };
        });
        descriptors.push(descriptor);
    }


    // Cleanup
    for _ in 0..number_of_nodes {
        sender_stop.send(Signals::Terminate).unwrap();
    }
    for thread in descriptors {
        thread.join().unwrap();
    }
}


#[cfg(test)]
mod tests {
    use crate::parse_arguments;
    use std::sync::*;
    use crate::Signals;
    use crate::terminate;

    #[test]
    fn command_line_parsing() {
        let negative_number = vec![-1];
        let zero = vec![0];
        let positive = vec![1];
        assert_eq!(parse_arguments(negative_number.into()), -negative_number);
        assert_eq!(parse_arguments(zero.into()), 1);
        assert_eq!(parse_arguments(positive.into()), positive);
    }

    #[test]
    fn terminate_thread() {
        let (sender_stop, recv_stop) = mpsc::channel();
        let recv_stop = Arc::new(Mutex::new(rx_stop));
        sender_stop.send(Signals::Terminate);

        assert_eq!(terminate(recv_stop), true);
    }
}
