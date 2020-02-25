use crate::block::{self, *};
use crate::crypto::hash::{H256,Hashable};
use crate::crypto::hash;
use std::collections::HashMap;

pub struct Blockchain {
    pub chain:HashMap<H256,Block>,
    pub tiphash:H256,
    pub heights:HashMap<H256,u8>,
    pub buffer:HashMap<H256,Block>,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let mut buffer: [u8; 32] = [0; 32];
        let b:H256 = buffer.into();
        let mut genesis:Block = block::generate_genesis_block(&b);
        let mut genhash:H256 = genesis.hash();
        let mut chainmap:HashMap<H256,Block> = HashMap::new();
        let mut heightsmap:HashMap<H256,u8> = HashMap::new();
        let mut buffermap:HashMap<H256,Block> = HashMap::new();
        chainmap.insert(genhash,genesis);
        heightsmap.insert(genhash,0);
        let t:H256 = genhash;
        let mut newchain:Blockchain = Blockchain{chain:chainmap,tiphash:t,heights:heightsmap,buffer:buffermap};
        newchain
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {


        let h:H256 = block.hash();
        let mut flag:bool = false;


        match self.heights.get(&block.header.parenthash){
            Some(&number) => { //insertion into mainchain
                self.chain.insert(h,block.clone());
                let len = self.heights[&block.header.parenthash]+1;
                self.heights.insert(h,len);
                if(len>self.heights[&self.tiphash]){
                    self.tiphash = h;
                }

                let mut bhash_copy:H256 = hash::generate_random_hash();
                //if stale blocks parent has arrived, insert it into main chain
                for (bhash,blck) in self.buffer.iter(){
                    if blck.header.parenthash == h {
                        flag = true;
                        bhash_copy = *bhash;
                        self.chain.insert(h,blck.clone());
                        let len = self.heights[&blck.header.parenthash]+1;
                        self.heights.insert(h,len);
                        if(len>self.heights[&self.tiphash]){
                            self.tiphash = h;
                        }
                    }
                }
                if flag {
                self.buffer.remove(&bhash_copy);
                }
            }, // insert stale block into buffer
            _ => { self.buffer.insert(h,block.clone()); },
        }

    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tiphash
    }

    /// Get the last block's hash of the longest chain
    #[cfg(any(test, test_utilities))]
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
