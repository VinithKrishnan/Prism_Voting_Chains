use crate::network::server::Handle as ServerHandle;
use crate::block::{self, *};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::merkle::MerkleTree;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;

pub struct SB_header {
    // pub parent_mkl_root: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: i64,
    pub content_mkl_root: H256,
    pub miner_id: i32,
}

pub struct Superblock {
    pub header: SB_header,
    pub content: Vec<Content>,
}

pub fn get_difficulty() -> H256 {
    (hex!("0000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")).into()
}

impl Hashable for Superblock {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Hashable for SB_header {
    fn hash(&self) -> H256 {
        let encoded = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &encoded).into()
    }
}

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
    blockchain: Arc<Mutex<Blockchain>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
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

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }

                loop {
                    // step1: assemble a new superblock
                    // TODO: We can optimize the assembly by using the version numbers trick
                    let locked_blockchain = self.blockchain.lock().unwrap();
                    // let mut parents: Vec<H256> = Vec::new();
                    let mut contents: Vec<Content> = Vec::new();

                    // create a proposer block
                    let proposer_content = ProposerContent {
                        parent_hash: locked_blockchain.get_proposer_tip(),
                        transactions: vec![],
                        proposer_refs: locked_blockchain.get_unref_proposers(),
                    };
                    // parents.push(proposer_content.parent_hash);
                    contents.push(proposer_content);

                    // create all voter blocks
                    let num_voter_chains = locked_blockchain.num_voter_chains;
                    for chain_num in 1..(num_voter_chains + 1) {
                        let tmp = VoterContent {
                            votes: locked_blockchain.get_votes(chain_num),
                            parent_hash: locked_blockchain.get_voter_tip(chain_num),
                            chain_num: chain_num,
                        }
                        // parents.push(tmp.parent_hash);
                        contents.push(tmp);
                    }

                    drop(locked_blockchain);

                    // let parent_mkl_tree = MerkleTree::new(&parents);
                    let content_mkl_tree = MerkleTree::new(&contents);

                    // let sb_content = SB_content {
                    //     parents: parents,
                    //     contents: contents,
                    // }

                    let mut rng = rand::thread_rng();

                    let sb_header = SB_header {
                        // parent_mkl_root: parent_mkl_tree.root(),
                        nonce: rng.gen::<u32>(),
                        difficulty: get_difficulty(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                        content_mkl_root: content_mkl_tree.root(),
                        miner_id: 0,    // TODO: set proper miner ID
                    }

                    let superblock = Superblock {
                        header: sb_header,
                        content: contents,
                    }

                    if superblock.hash() < get_difficulty() {
                        // TODO: sort into a block type, create a processed block, include sortition proof
                        // TODO: insert into the blockchain
                        // TODO: broadcast to the network
                    }

                }


            }
        }
    }
}