use crate::block::{self, *};
use crate::crypto::hash::{H256,Hashable};
use log::debug;
use std::collections::HashMap;
use std::collections::VecDeque;
use crate::mempool::TransactionMempool;
use crate::ledger_state::{BlockState,update_block_state};
use crate::utils::{*};
use crate::crypto::address::H160;

extern crate chrono;
use chrono::prelude::*;

pub struct Blockchain {
    pub chain:HashMap<H256,Block>,
    pub tiphash:H256,
    pub heights:HashMap<H256,u8>,
    pub buffer:HashMap<H256,Block>,
    pub totaldelay:i64,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let buffer: [u8; 32] = [0; 32];
        let b:H256 = buffer.into();
        let genesis:Block = block::generate_genesis_block(&b);
        let genhash:H256 = genesis.hash();
        let mut chainmap:HashMap<H256,Block> = HashMap::new();
        let mut heightsmap:HashMap<H256,u8> = HashMap::new();
        let buffermap:HashMap<H256,Block> = HashMap::new();
        chainmap.insert(genhash,genesis);
        heightsmap.insert(genhash,0);
        let t:H256 = genhash;
        let newchain:Blockchain = Blockchain{chain:chainmap,tiphash:t,heights:heightsmap,buffer:buffermap,totaldelay:0};
        newchain
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block,mut mempool:&mut TransactionMempool,mut blockstate:&mut BlockState) {


        let h:H256 = block.hash();
        //let mut flag:bool = false;


        match self.chain.get(&block.header.parenthash){
            Some(pblock) => { //insertion into mainchain
                let mut validity:bool = false;
                if h < pblock.header.difficulty && !self.chain.contains_key(&h) {
                let b_delay = Local::now().timestamp_millis() - block.header.timestamp;
                self.totaldelay = self.totaldelay + b_delay;
                if blockstate.block_state_map.contains_key(&block.header.parenthash){
                validity = is_blck_valid(&block,&blockstate.block_state_map.get(&block.header.parenthash).unwrap());
                }
                else {
                println!("State of parent block not found");
                return;
                }
                //checks and updates
                if !validity {
                    println!("Block with hash {} does not satisfy state validity",h);
                    return;
                }
                

                println!("Adding block with hash {} mined by  node {} to chain",h,block.header.miner_id);
                println!("Block delay is: {:?}",(Local::now().timestamp_millis() - block.header.timestamp));
                println!("Average delay is {}",self.totaldelay/(self.chain.len() as i64));
                println!("Total number of blocks in blockchain:{}\n",self.chain.len());
                self.chain.insert(h,block.clone());
                mempool_update(&block,&mut mempool);
                update_block_state(&block,&mut blockstate);
                
                let len = self.heights[&block.header.parenthash]+1;
                self.heights.insert(h,len);
                if len>self.heights[&self.tiphash] {
                    self.tiphash = h;
                    println!("Current tipheight is {}",len);
                    println!("All blocks in longest chain: {:?}",self.all_blocks_in_longest_chain());
                    

                    let mut temp_state_map = blockstate.block_state_map.get(&block.hash()).unwrap();
                    let mut utxo_hmap:HashMap<H160,u32> = HashMap::new();

                    for (utxo_input,utxo_output) in temp_state_map.state_map.iter() {
                        if !utxo_hmap.contains_key(&utxo_output.receipient_addr){
                            utxo_hmap.insert(utxo_output.receipient_addr,utxo_output.value);
                        }else{
                            *utxo_hmap.get_mut(&utxo_output.receipient_addr).unwrap() = *utxo_hmap.get_mut(&utxo_output.receipient_addr).unwrap()+utxo_output.value;
                        }
                    }
                    for (key,value) in utxo_hmap.iter() {
                        println!("balance in addr {:?} is {:?}",key,value);
                    }
                }

                //let mut bhash_copy:H256 = hash::generate_random_hash();
                //if stale blocks parent has arrived, insert it into main chain
                let mut bhash_vec = Vec::new();
                let mut phash_q: VecDeque<H256>= VecDeque::new();
                phash_q.push_back(h);
                while !phash_q.is_empty() {
                    match phash_q.pop_front() {
                        Some(h) => for (bhash,blck) in self.buffer.iter(){
                                if blck.header.parenthash == h {
                                    //flag = true;
                                    let bhash_copy:H256 = *bhash;
                                    bhash_vec.push(bhash_copy);
                                    //checks and updates
                                    let mut validity:bool = false;
                                    if blockstate.block_state_map.contains_key(&h){
                                        validity = is_blck_valid(&block,&blockstate.block_state_map.get(&h).unwrap());
                                    }
                                    else {
                                        println!("State of parent block not found");
                                        continue;
                                    }
                                    //let validity:bool = is_blck_valid(&block,&blockstate.block_state_map.get(&block.header.parenthash).unwrap());
                                        if !validity {
                                        println!("Block with hash {} does not satisfy state validity",h);
                                        return;
                                        }
                                    
                                

                                    self.chain.insert(bhash_copy,blck.clone());
                                    mempool_update(&blck,&mut mempool);
                                    update_block_state(&blck,&mut blockstate);
                                    let b_delay = Local::now().timestamp_millis() - blck.header.timestamp;
                                    self.totaldelay = self.totaldelay + b_delay;

                                    println!("Adding block with hash {} to chain",blck.hash());
                                    println!("Block delay is: {:?}",(Local::now().timestamp_millis() - blck.header.timestamp));
                                    println!("Average delay is {}",self.totaldelay/(self.chain.len() as i64));
                                    println!("Total number of blocks in blockchain:{}\n",self.chain.len());
                                    let len = self.heights[&blck.header.parenthash]+1;
                                    self.heights.insert(bhash_copy,len);
                                    if len>self.heights[&self.tiphash] {
                                        self.tiphash = bhash_copy;
                                    }
                                }
                            },
                        None => (),
                    }
                }


                for bh in bhash_vec{
                    self.buffer.remove(&bh);
                }
             }
            }, // insert stale block into buffer
            _ => {
                  print!("Adding block with hash {} to buffer\n",h); 
                  if !self.buffer.contains_key(&h){
                  self.buffer.insert(h,block.clone()); 
                  }
                 },
        }

    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tiphash
    }

    /// Get the last block's hash of the longest chain
    //#[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {

        let mut phash:H256 = self.tiphash;
        let mut result:Vec<H256>=vec![];
        let mut buffer: [u8; 32] = [0; 32];
        let b:H256 = buffer.into();
        while(phash!=b){
            result.push(phash);
            phash = self.chain[&phash].header.parenthash;
        }
        let mut res = result.reverse();
        result
    }
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = block::generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }
}
