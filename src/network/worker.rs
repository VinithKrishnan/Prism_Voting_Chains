// use super::buffer::BlockBuffer;
use super::message::Message;
use super::peer;
use crate::network::server::Handle as ServerHandle;
use crate::blockchain::{Blockchain, InsertStatus};
use crate::block::*;
use crate::transaction::SignedTransaction;
use crate::utils;
use crate::mempool::TransactionMempool;
use crate::crypto::hash::{H256, Hashable};
use std::collections::{HashMap, HashSet};
use crate::validation::{BlockResult,check_pow_sortition_id,check_sortition_proof};
use crossbeam::channel;
use log::{info,debug, warn};
use crate::validation::{self};

use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    tx_mempool: Arc<Mutex<TransactionMempool>>,
    requested_blcks: Arc<Mutex<HashSet<H256>>>,
    process_blck: Arc<Mutex<HashSet<H256>>>,
    requested_txs: Arc<Mutex<HashSet<H256>>>,
    process_txs:Arc<Mutex<HashSet<H256>>>,
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
    tx_mempool: &Arc<Mutex<TransactionMempool>>,
) -> Context {
    Context {
        msg_chan: msg_src,
        num_worker,
        server: server.clone(),
        blockchain: Arc::clone(blockchain),
        tx_mempool: Arc::clone(tx_mempool),
        requested_blcks: Arc::new(Mutex::new(HashSet::new())),
        process_blck: Arc::new(Mutex::new(HashSet::new())),
        requested_txs:Arc::new(Mutex::new(HashSet::new())),
        process_txs:Arc::new(Mutex::new(HashSet::new())),
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
            let mut locked_mempool = self.tx_mempool.lock().unwrap();
            match msg {
                Message::Ping(nonce) => {
                    println!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    println!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(vec_hashes) => {
                    let mut required_blocks:Vec<H256> = vec![];
                    println!("Received New Block Hashes");

                    for recv_hash in vec_hashes {
                        let mut flag: bool = false;

                        //check if hash value exit in proposer_chain
                        //TODO: write a contain func in blcokchain.rs
                        for (bhash,_) in locked_blockchain.proposer_chain.iter(){
                            if *bhash == recv_hash{
                                println!("Block that hashes to {} already present", bhash);
                                flag = true;
                            }
                        }
                        for voter_chain in &locked_blockchain.voter_chains{
                            for (bhash,_) in voter_chain.iter(){
                                if *bhash == recv_hash{
                                    println!("Block that hashes to {} already present", bhash);
                                    flag = true;
                                }
                            }
                        }
                        let requested_blcks = self.requested_blcks.lock().unwrap();
                        if requested_blcks.contains(&recv_hash) {
                            flag = true;
                        }
                        if !flag {
                            required_blocks.push(recv_hash);
                        }
                        drop(requested_blcks);
                    }
                    if required_blocks.len()!= 0 {
                        let mut requested_blcks = self.requested_blcks.lock().unwrap();
                        for hash in &required_blocks {
                            requested_blcks.insert(*hash);
                        }
                        println!("Sending getBlocks Message");
                        peer.write(Message::GetBlocks(required_blocks));
                        drop(requested_blcks);
                    }
                    

                }
                //TODO: might have to look into buffer too.
                Message::GetBlocks(vec_hashes) => {
                    let mut give_blocks:Vec<Block> = vec![];
                    println!("Received GetBlocks");
                    for recv_hash in vec_hashes {

                        //find blocks from proposer_chain and voter_chains
                        for (bhash,metablock) in locked_blockchain.proposer_chain.iter(){
                            if *bhash == recv_hash{
                                println!("Adding proposer block with hash {} to give_blocks", bhash);
                                give_blocks.push(metablock.block.clone());
                            }
                        }
                        for voter_chain in &locked_blockchain.voter_chains{
                            for (bhash,metablock) in voter_chain.iter(){
                                if *bhash == recv_hash{
                                    println!("Adding voter block with hash {} to give_blocks", bhash);
                                    give_blocks.push(metablock.block.clone());
                                }
                            }
                        }
                    }
                    if give_blocks.len()!=0 {
                        println!("Sending Blocks message");
                        peer.write(Message::Blocks(give_blocks));
                    }

                }
                Message::Blocks(vec_blocks) => {
                    println!("Received Blocks message");
                    let mut get_block_hash : Vec<H256> = vec![];
                    let mut new_block_hash : Vec<H256> = vec![];
                    for blck in vec_blocks{
                        let hash = blck.hash();
                        let mut requested_blcks = self.requested_blcks.lock().unwrap();
                        requested_blcks.remove(&hash);
                        drop(requested_blcks);
                        //check if the block is being processed
                        let mut process_blck = self.process_blck.lock().unwrap();
                        if process_blck.contains(&hash){
                            drop(process_blck);
                            continue;
                        }
                        process_blck.insert(hash);
                        drop(process_blck);
                        //pow and sortation id
                        /*let pow_sor_check = check_pow_sortition_id(&blck,&locked_blockchain);
                        match pow_sor_check{
                            BlockResult::Fail => {
                                continue;
                            }
                            _ => {}
                        }*/
                        //sortition proof
                        /*let sor_proof_check = check_sortition_proof(&blck,&locked_blockchain);
                        match sor_proof_check{
                            BlockResult::Fail => {
                                continue;
                            }
                            _ => {}

                        }*/
                        //check_content_semantic
                        /*let content_check = validation::check_content_semantic(&block,&locked_blockchain);
                        if content_check.is_none {
                            println!("Sortition proof failed");
                            continue;
                        }*/
                        //insert here
                        let insert_status = locked_blockchain.insert(&blck);
                        match insert_status {
                            InsertStatus::Orphan => {
                                match blck.content.clone() {
                                    Content::Proposer(content) => {
                                        get_block_hash.push(content.parent_hash);
                                    }
                                    Content::Voter(content) => {
                                        get_block_hash.push(content.parent_hash);
                                    }
                                }
                                //get_block_hash.push(blck.content.parent_hash);
                                self.server.broadcast(Message::GetBlocks(get_block_hash.clone()));
                            }
                            _ => {}
                        }
                        //broadcasting new block hashes
                        new_block_hash.push(blck.clone().hash());
                        self.server.broadcast(Message::NewBlockHashes(new_block_hash.clone()));
                    }
                }
                Message::NewTransactionHashes(vec_tx_hashes) => {
                    let mut required_txs: Vec<H256> = vec![];
                    println!("Received NewTransactionHashes");
                    //locked_mempool = self.tx_mempool.lock().unwrap();
                    for recv_tx_hash in vec_tx_hashes {
                        if !locked_mempool.contains(&recv_tx_hash) {
                            let requested_txs = self.requested_txs.lock().unwrap();
                            if !requested_txs.contains(&recv_tx_hash) {
                                required_txs.push(recv_tx_hash.clone());
                            }
                           drop(requested_txs);
                           // drop(locked_mempool);
                        } else {
                            println!("tx which hashes to {} already present in mempool",recv_tx_hash)
                        }
                    }

                    if required_txs.len()!= 0 {
                        println!("Sending GetTransactions Message");
                        peer.write(Message::GetTransactions(required_txs));
                    }
                }
                Message::GetTransactions(vec_tx_hashes) => {
                    let mut txs_to_send:Vec<SignedTransaction> = vec![];
                    println!("Received GetTransactions");

                    for tx_hash in vec_tx_hashes {
                        match locked_mempool.get(&tx_hash){
                            Some(txstore) => txs_to_send.push(txstore.signed_tx.clone()),
                            None => println!("tx which hashes to {} not present in mempool", tx_hash),
                        }
                    }

                    if txs_to_send.len()!=0 {
                        println!("Sending Transactions message");
                        peer.write(Message::Transactions(txs_to_send));
                    }
                }
                Message::Transactions(vec_signed_txs) => {
                    println!("Received Transactions");
                    let mut tx_hashes_to_broadcast: Vec<H256> = vec![];
                    for signed_tx in vec_signed_txs {
                        let mut requested_txs = self.requested_txs.lock().unwrap();
                        requested_txs.remove(&signed_tx.hash());
                        drop(requested_txs);
                        let signed_tx_hash = signed_tx.hash();
                        match locked_mempool.get(&signed_tx_hash){
                          Some(_tx_present) => println!("tx_hash {} already present. Not adding to mempool",
                                                     signed_tx_hash),
                          None => {
                              println!("tx_hash {} is being added to mempool", signed_tx_hash);
                              locked_mempool.insert(signed_tx);
                              tx_hashes_to_broadcast.push(signed_tx_hash);
                          }
                        }
                    }
                    if tx_hashes_to_broadcast.len() != 0{
                        self.server.broadcast(Message::NewTransactionHashes(tx_hashes_to_broadcast.clone()));
                    }
                }
            }
        //std::mem::drop(locked_state);
        std::mem::drop(locked_mempool);
        std::mem::drop(locked_blockchain);

        }
    }
}
