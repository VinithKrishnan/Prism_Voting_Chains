use crate::network::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::mempool::TransactionMempool;
use crate::ledger_state::BlockState;
use crate::block::*;
use crate::transaction::{self, SignedTransaction};
use crate::crypto::hash::{H256, Hashable,generate_random_hash};
use crate::crypto::merkle::{*};
use crate::network::message::Message;
use log::{info,debug};
use rand::Rng;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};

extern crate chrono;
use chrono::prelude::*;

use std::time;
use std::thread;
use std::sync::{Arc, Mutex};

enum ControlSignal {
    Start(u64,u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64,u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool:Arc<Mutex<TransactionMempool>>,
    num_mined:u8,
    ledger_state: Arc<Mutex<BlockState>>,
    header:Header,
    contents: Vec<Content>,
    content_merkle_tree: MerkleTree,
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
    ledger_state: &Arc<Mutex<BlockState>>
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    
    let mut contents: Vec<Content> = vec![];
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        mempool: Arc::clone(mempool),
        num_mined:0,
        ledger_state: Arc::clone(ledger_state),
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

    pub fn start(&self, lambda: u64,index: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda,index))
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

    pub fn tx_pool_gen(&self,mempool:&mut TransactionMempool) -> Content {
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
        println!("The len of mempool is {}",mempool.tx_hash_queue.len());
        while vect.len()<1 && mempool.tx_hash_queue.len()>0 {
        let h = mempool.tx_hash_queue.pop_front().unwrap();
        match mempool.tx_to_process.get(&h) {
            Some(boolean) => if *boolean && mempool.tx_map.contains_key(&h){
                vect.push(mempool.tx_map.get(&h).unwrap().clone());
                //merkle_init_vect.push(h);
            },
            None => continue
            
        }
       }
        
        
        let mut content: Content = Content{data:vect};
        //let mut merkle_tree_tx = MerkleTree::new(&merkle_init_vect);
        //let mut merkle_root = merkle_tree_tx.root();
    
        content
    
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                println!("Miner shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i,j) => {
                println!("Miner starting in continuous mode with lambda {} adn index {}", i , j);
                self.operating_state = OperatingState::Run(i,j);
            }
        }
    }

    fn miner_loop(&mut self) {
        let mut flag:bool = true;
        // main mining loop
        let mut content:Content;
        let mut vect: Vec<SignedTransaction> = vec![];
        content = Content{data:vect};
        let mut merkle_root:H256=generate_random_hash();
        let mut  i:u32 = 0;
        loop {
            let mut index:u64 = 0;
            let mut time_i:u64 = 0;
            //println!("Inside mining loop");
            
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

            if let OperatingState::Run(i,j) = self.operating_state {
                index = j;
                time_i =i;  
            }

             
            // transaction pool generation for block 
            /*
            let mut vect: Vec<SignedTransaction> = vec![];
            let mut merkle_init_vect: Vec<H256> = vec![];

            println!("About to enter loop");
            loop {
            let mut locked_mempool = self.mempool.lock().unwrap();
            if locked_mempool.transq.len()<15 {
                std::mem::drop(locked_mempool);
                continue;
            } else {
                while vect.len()<10 && locked_mempool.transq.len()>0 {
                    let h = locked_mempool.transq.pop_front().unwrap();
                    if locked_mempool.bool_map.contains_key(&h) && locked_mempool.bool_map.get(&h).unwrap() == &true {
                        vect.push(locked_mempool.trans_map.get(&h).unwrap().clone());
                        merkle_init_vect.push(h);
                    }
                }
                std::mem::drop(locked_mempool);
                if vect.len()==10 {
                break;
                }
            }
            }
            let mut content: Content = Content{data:vect};
            let mut merkle_tree_tx = MerkleTree::new(&merkle_init_vect);
            let mut merkle_root = merkle_tree_tx.root();
            */
           
            let mut locked_blockchain = self.blockchain.lock().unwrap();
            let mut locked_mempool = self.mempool.lock().unwrap();
            let mut locked_state = self.ledger_state.lock().unwrap();
            
        
            if flag {
                content = self.tx_pool_gen(&mut locked_mempool);
            }
            flag = false;
            
            // actual mining
            //println!("Out of loop");
            // create Block
            //TODO: Put this in a function

            //Creating Header fields
            
            let mut merkle_init_vect: Vec<H256> = vec![];
            if content.data.len()==0{
                continue;
            }
            
            
             
            for tx in content.data.iter() {
             merkle_init_vect.push(tx.hash());
            }
            let mut merkle_tree_tx = MerkleTree::new(&merkle_init_vect);
            let mut merkle_root = merkle_tree_tx.root();
           
            let phash = locked_blockchain.tiphash;

            let mut rng = rand::thread_rng();
            let nonce = rng.gen();

            let timestamp = Local::now().timestamp_millis();
            let mut difficulty:H256 =  hex!("09911718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920").into();
            
            if locked_blockchain.chain.contains_key(&locked_blockchain.tiphash){
            difficulty = locked_blockchain.chain.get(&locked_blockchain.tiphash)
                             .unwrap()
                             .header
                             .difficulty ;
            }

            //Creating Content
            //It will also be used for Merkel Root for the Header
            /*let t = transaction::generate_random_signed_transaction();
            let mut vect: Vec<SignedTransaction> = vec![];
            vect.push(t);
            let content: Content = Content{data:vect};

            let merkle_root = H256::from([0; 32]);*/

            //Putting all together
            let header = Header{
                parenthash: phash,
                nonce: nonce,
                difficulty: difficulty,
                timestamp: timestamp,
                merkle_root: merkle_root,
                miner_id:index as i32,
            };
            let new_block = Block{header: header, content: content.clone()};
            //Check whether block solved the puzzle
            //If passed, add it to blockchain
            
            if new_block.hash() <= difficulty {
              println!("block with hash:{} generated\n",new_block.hash());
              println!("Number of blocks mined until now:{}\n",self.num_mined+1);
              println!("Block generated by node {}",new_block.header.miner_id);
              locked_blockchain.insert(&new_block,&mut locked_mempool,&mut locked_state);
              let encodedhead: Vec<u8> = bincode::serialize(&new_block).unwrap();
              println!("Size of block generated is {} bytes\n",encodedhead.len());
              //print!("Total number of blocks in blockchain:{}\n",locked_blockchain.chain.len());
              self.num_mined = self.num_mined + 1;
              let mut new_block_hash : Vec<H256> = vec![];
              new_block_hash.push(new_block.hash());
              self.server.broadcast(Message::NewBlockHashes(new_block_hash));
              
              content = self.tx_pool_gen(&mut locked_mempool);
              
              for tx in content.data.iter() {
                merkle_init_vect.push(tx.hash());
               }
               merkle_tree_tx = MerkleTree::new(&merkle_init_vect);
               merkle_root = merkle_tree_tx.root();
            }

            std::mem::drop(locked_state);
            std::mem::drop(locked_mempool);
            std::mem::drop(locked_blockchain);
            

            /*if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }*/

            if time_i != 0 {
                let interval = time::Duration::from_micros(time_i);
                thread::sleep(interval);
            }
        }
    }
}
