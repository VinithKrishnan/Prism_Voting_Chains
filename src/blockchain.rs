use crate::block::*;
use crate::crypto::hash::{H256,Hashable};
use std::collections::HashMap;
use crate::block::test::generate_random_block;

pub struct Blockchain {
    chain:HashMap<H256,Block>,
    tiphash:H256,
    heights:HashMap<H256,u8>,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let mut buffer: [u8; 32] = [0; 32];
        let b:H256 = buffer.into();
        let mut genesis:Block = generate_random_block(&b);
        let mut genhash:H256 = genesis.hash();
        let mut chainmap:HashMap<H256,Block> = HashMap::new();
        let mut heightsmap:HashMap<H256,u8> = HashMap::new();
        chainmap.insert(genhash,genesis);
        heightsmap.insert(genhash,0);
        let t:H256 = genhash;
        let mut newchain:Blockchain = Blockchain{chain:chainmap,tiphash:t,heights:heightsmap};
        newchain
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let h:H256 = block.hash();
        self.chain.insert(h,block.clone());
        let len = self.heights[&block.header.parenthash]+1;
        self.heights.insert(h,len);
        if(len>self.heights[&self.tiphash]){
            self.tiphash = h;
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
    use crate::block::test::generate_random_block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }
}
