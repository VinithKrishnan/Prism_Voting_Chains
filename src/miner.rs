use crate::network::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::block::*;
use crate::transaction::{self, Transaction};
use crate::crypto::hash::{H256, Hashable};

use log::info;
use rand::Rng;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};

use std::time::{self, SystemTime, UNIX_EPOCH};
use std::thread;
use std::sync::{Arc, Mutex};

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain)
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i) => {
                info!("Miner starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

    fn miner_loop(&mut self) {
        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // actual mining
            
            // create Block
            //TODO: Put this in a function
            
            //Creating Header fields
            let mut locked_blockchain = self.blockchain.lock().unwrap();
            let phash = locked_blockchain.tiphash;

            let mut rng = rand::thread_rng();
            let nonce = rng.gen();
            
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            let difficulty = locked_blockchain.chain.get(&locked_blockchain.tiphash)
                             .unwrap()
                             .header
                             .difficulty ;
            
            //Creating Content
            //It will also be used for Merkel Root for the Header
            let t = transaction::generate_random_transaction();
            let mut vect: Vec<Transaction> = vec![];
            vect.push(t);
            let content: Content = Content{data:vect};

            let merkle_root = H256::from([0; 32]); 

            //Putting all together
            let header = Header{
                parenthash: phash,
                nonce: nonce,
                difficulty: difficulty,
                timestamp: timestamp,
                merkle_root: merkle_root
            };
            let new_block = Block{header: header, content: content};
            
            //Check whether block solved the puzzle
            //If passed, add it to blockchain
            if new_block.hash() <= difficulty {
              print!("block is generated\n");
              locked_blockchain.insert(&new_block);
            }

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }

            std::mem::drop(locked_blockchain);
        }
    }
}
