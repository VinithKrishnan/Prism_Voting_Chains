use crate::network::server::Handle as ServerHandle;
use crate::block::{self, *};
use crate::blockchain::{Blockchain};
use crate::crypto::hash::{H256, Hashable};
use crate::mempool::{TransactionMempool};
use crate::crypto::merkle::MerkleTree;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use crate::network::message::{Message};
use log::info;
use bigint::uint::U256;
use rand::Rng;
use crate::transaction::{self, SignedTransaction};

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;

const TOTAL_SORTITION_WIDTH: u64 = std::u64::MAX;
pub const PROPOSER_INDEX: u32 = 0;
pub const FIRST_VOTER_IDX: u32 = 1;

pub struct Superblock {
    pub header: Header,
    pub content: Vec<Content>,
}

impl Hashable for Superblock {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

pub fn get_difficulty() -> H256 {
    (hex!("0000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")).into()
}

pub fn sortition_hash(hash: H256, difficulty: H256, num_voter_chains: u32) -> Option<u32> {
    let hash = U256::from_big_endian(hash.as_ref());
    let difficulty = U256::from_big_endian(difficulty.as_ref());
    let multiplier = difficulty / TOTAL_SORTITION_WIDTH.into();
    
    let precise: f32 = (1.0 / (num_voter_chains + 1) as f32) * TOTAL_SORTITION_WIDTH as f32;
    let proposer_sortition_width: u64 = precise.ceil() as u64;
    let proposer_width = multiplier * proposer_sortition_width.into();
    if hash < proposer_width {
        Some(PROPOSER_INDEX)
    } else if hash < difficulty {
        let voter_idx = (hash - proposer_width) % num_voter_chains.into();
        Some(FIRST_VOTER_IDX + voter_idx.as_u32())
    } else {
        println!("Why you sortitioning something that is not less than difficulty?");
        None
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
    mempool:Arc<Mutex<TransactionMempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<TransactionMempool>>,  
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool)
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

    pub fn tx_pool_gen(&self,mempool:&mut TransactionMempool) -> Vec<SignedTransaction> {
        let mut vect: Vec<SignedTransaction> = vec![];
        //let mut merkle_init_vect: Vec<H256> = vec![];
    
        
        //for k in 1..6 {
        
       
        //let mut locked_mempool = self.mempool.lock().unwrap();
        /*
        if mempool.tx_hash_queue.len()<15 {
            continue;
        } else {
            while vect.len()<10 && mempool.tx_hash_queue.len()>0 {
                let h = mempool.tx_hash_queue.pop_front().unwrap();
                if mempool.tx_to_process.contains_key(&h) && mempool.tx_to_process.get(&h).unwrap() == &true {
                    vect.push(mempool.tx_map.get(&h).unwrap().clone());
                    merkle_init_vect.push(h);
                }
            }
            if vect.len()==10 {
            break;
            }
        }*/
        println!("The len of mempool is {}",mempool.mempool_len());
        vect = mempool.get_transactions(1);
        //let mut merkle_tree_tx = MerkleTree::new(&merkle_init_vect);
        //let mut merkle_root = merkle_tree_tx.root();
        vect
    
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
                    let locked_mempool = self.blockchain.lock().unwrap();
                    let mut contents: Vec<Content> = Vec::new();
                    
                    let txs = vec![];
                    while txs.len() == 0 {
                        let locked_mempool = self.mempool.lock().unwrap();
                        txs = self.tx_pool_gen(&mut locked_mempool);
                        drop(locked_mempool);
                    }

                    //proposer
                    let proposer_content = ProposerContent {
                        parent_hash: locked_blockchain.get_proposer_tip(),
                        transactions: txs,
                        proposer_refs: locked_blockchain.get_unref_proposers(),
                    };
                    contents.push(block::Content::Proposer(proposer_content));

                    // Voters
                    let num_voter_chains = locked_blockchain.num_voter_chains;
                    for chain_num in 1..(num_voter_chains + 1) {
                        let tmp = VoterContent {
                            votes: locked_blockchain.get_votes(chain_num),
                            parent_hash: locked_blockchain.get_voter_tip(chain_num),
                            chain_num: chain_num,
                        };
                        contents.push(block::Content::Voter(tmp));
                    }

                    drop(locked_blockchain);

                    let content_mkl_tree = MerkleTree::new(&contents);

                    let mut rng = rand::thread_rng();
                    let header = Header {
                        nonce: rng.gen::<u32>(),
                        difficulty: get_difficulty(),
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                        merkle_root: content_mkl_tree.root(),
                        miner_id: 0,    // TODO: set proper miner ID
                    };

                    let superblock = Superblock {
                        header: header,
                        content: contents,
                    };

                    let block_hash = superblock.hash();
                    // NOTE: Below works only for static difficulty
                    let difficulty = get_difficulty();

                    if block_hash < difficulty {
                        println!("Mined a new block");
                        // Sortition and decide the block index - proposer(0), voters(1..m)
                        let block_idx: u32 = sortition_hash(block_hash, difficulty, num_voter_chains).unwrap();

                        // Add header, relevant content and sortition proof
                        let sortition_proof = content_mkl_tree.proof(block_idx as usize);
                        let processed_block = Block {
                            header: superblock.header,
                            content: superblock.content[block_idx as usize],
                            sortition_proof: sortition_proof,
                        };

                        // Insert into local blockchain
                        let locked_blockchain = self.blockchain.lock().unwrap();
                        locked_blockchain.insert(&processed_block);
                        drop(locked_blockchain);

                        // Broadcast new block hash to the network
                        self.server.broadcast(Message::NewBlockHashes(vec![block_hash]));
                    }
                }
            }
        }
    }
}