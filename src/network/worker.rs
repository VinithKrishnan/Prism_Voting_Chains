use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crossbeam::channel;
use log::{debug, warn};
use crate::blockchain::Blockchain;
use crate::block::*;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::crypto::hash::{H256, Hashable};

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>
) -> Context {
    Context {
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
    }
}

impl Context {
    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let msg = self.msg_chan.recv().unwrap();
            let (msg, peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            let mut locked_blockchain = self.blockchain.lock().unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(vec_hashes) => {
                    let mut required_blocks:Vec<H256> = vec![];
                    debug!("Received New Block Hashes");

                    let mut flag:bool = false;
                    for recv_hash in vec_hashes {
                        flag = false;
                        for (bhash,blck) in locked_blockchain.chain.iter(){
                            if *bhash == recv_hash{
                                debug!("Block that hashes to {} already present", bhash);
                                flag = true;
                            }
                        }
                        for (bhash,blck) in locked_blockchain.buffer.iter(){
                            if *bhash == recv_hash {
                                debug!("Block that hashes to {} already present", bhash);
                                flag = true;
                            }
                        }
                        if !flag {
                            required_blocks.push(recv_hash);
                        }
                    }
                    if required_blocks.len()!= 0 {
                        debug!("Sending getBlocks Message");
                        peer.write(Message::GetBlocks(required_blocks));
                    }

                }
                Message::GetBlocks(vec_hashes) => {
                    let mut give_blocks:Vec<Block> = vec![];
                    debug!("Received GetBlocks");
                    for getblock_hash in vec_hashes {
                        for (bhash,blck) in locked_blockchain.chain.iter(){
                            if *bhash == getblock_hash{
                                debug!("Adding block with hash {} to give_blocks", bhash);
                                give_blocks.push(blck.clone());
                            }
                        }
                        for (bhash,blck) in locked_blockchain.buffer.iter(){
                            if *bhash == getblock_hash {
                                debug!("Adding block with hash {} to give_blocks", bhash);
                                give_blocks.push(blck.clone());
                            }
                        }

                    }
                    if give_blocks.len()!=0 {
                        debug!("Sending Blocks message");
                        peer.write(Message::Blocks(give_blocks));
                    }

                }
                Message::Blocks(vec_blocks) => {
                    for blck in vec_blocks {
                        // added difficulty check in insert method
                        locked_blockchain.insert(&blck);
                        
                        //Sending getblocks message if block is orphan
                        let mut get_block_hash : Vec<H256> = vec![];
                        get_block_hash.push(blck.header.parenthash);
                        if !locked_blockchain.chain.contains_key(&blck.header.parenthash){
                            self.server.broadcast(Message::GetBlocks(get_block_hash));
                        }

                        //broadcasting NewBlockHashes
                        let mut new_block_hash : Vec<H256> = vec![];
                        new_block_hash.push(blck.hash());
                        self.server.broadcast(Message::NewBlockHashes(new_block_hash));
                    }
                }
            }
        }
    }
}
