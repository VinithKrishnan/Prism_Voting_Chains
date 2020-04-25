use crate::block::{self, *};
use crate::crypto::hash::{H256,Hashable};
use log::debug;
use std::collections::HashMap;
use std::collections::VecDeque;
use crate::utils::{*};

extern crate chrono;
use chrono::prelude::*;
use std::cmp;


pub enum InsertStatus {
    // Invalid,
    Orphan,
    Valid
}

pub struct Metablock {
    pub block: Block,
    pub level: u32,
}

pub struct Blockchain {
    pub proposer_chain: HashMap<H256, Metablock>,
    pub proposer_tip: H256,
    pub proposer_depth: u32,

    pub voter_chains: Vec<HashMap<H256, Metablock>>,
    pub voter_tips: Vec<H256>,
    pub voter_depths: Vec<u32>,

    // M: list of unreferenced proposer blocks
    pub unref_proposers: Vec<H256>,
    // M: Hash of first proposer block seen corresponding to each level
    pub level2proposer: HashMap<u32, H256>,
    // LM: level -> proposer hash mapping
    pub level2allproposers: HashMap<u32, Vec<H256>>,
    // LM: store the number of votes for each proposer
    pub proposer2votecount: HashMap<H256, u32>,

    // Last voted level corresponding to each voter chain
    // IMP TODO: need changes to handle forking in the voter chain
    // TODO: which size to use? u16 or u32
    pub chain2level: HashMap<u32, u32>,

    // orphan buffer stores a mapping between missing reference and block
    // use multimap as many blocks could wait on a single reference.
    pub orphan_buffer: HashMap<H256, Vec<Block>>,
}

impl Blockchain {
    pub fn new(m: u32) -> Self {
        // genesis for proposer and voter chains
        let mut proposer_chain = HashMap::new();
        let proposer = genesis_proposer();
        let proposer_hash = proposer.hash();
        let metablock = Metablock {
            block: proposer,
            level: 1,
        };
        proposer_chain.insert(proposer_hash, metablock);
        let proposer_tip = proposer_hash;

        let mut voter_chains = Vec::new();
        let mut voter_tips = Vec::new();
        let mut voter_depths = Vec::new();
        let mut chain2level = HashMap::new();
        for chain_num in 1..m {
            let mut tmp_chain = HashMap::new();
            let voter = genesis_voter(chain_num);
            let voter_hash = voter.hash();
            let metablock = Metablock {
                block: voter,
                level: 1,
            };
            tmp_chain.insert(voter_hash, metablock);
            voter_chains.push(tmp_chain);
            voter_tips.push(voter_hash);
            voter_depths.push(1);

            chain2level.insert(chain_num, 0);
        } 

        let mut unref_proposers = Vec::new();
        unref_proposers.push(proposer_hash);

        let mut level2proposer = HashMap::new();
        level2proposer.insert(1, proposer_hash);

        let mut level2allproposers = HashMap::new();
        level2allproposers.insert(1, vec![proposer_hash]);

        let mut proposer2votecount = HashMap::new();
        proposer2votecount.insert(proposer_hash, 0);

        Blockchain {
            proposer_chain: proposer_chain,
            proposer_tip: proposer_hash,
            proposer_depth: 1,

            voter_chains: voter_chains,
            voter_tips: voter_tips,
            voter_depths: voter_depths,

            unref_proposers: unref_proposers,
            level2proposer: level2proposer,
            level2allproposers: level2allproposers,

            proposer2votecount: proposer2votecount,
            chain2level: chain2level,

            orphan_buffer: HashMap::new(),
        }
    }

    pub fn is_orphan (&mut self, block: &Block) -> bool {
        // If there are missing references, it will add 
        // (first missing ref -> block) entry to orphan buffer map
        match block.content {
            Content::Proposer(content) => {
                if (!self.proposer_chain.contains_key(&block.header.parenthash)) {
                    // parent proposer not found, add to orphan buffer
                    self.orphan_buffer.entry(block.header.parenthash).or_insert(Vec::new()).push(block);
                    return true;
                }

                for ref_proposer in content.proposer_refs {
                    if (!self.proposer_chain.contains_key(&ref_proposer)) {
                        self.orphan_buffer.entry(ref_proposer).or_insert(Vec::new()).push(block);
                        return true;
                    }
                }
                return false;
            }
            Content::Voter(content) => {
                let chain_num = content.chain_num;

                if (!self.voter_chains[(chain_num-1) as usize].contains_key(&block.header.parenthash)) {
                    // parent proposer not found, add to orphan buffer
                    self.orphan_buffer.entry(content.parent_hash).or_insert(Vec::new()).push(block);
                    // self.orphan_buffer.insert(block.header.parenthash, block);
                    return true;
                }

                for vote in content.votes {
                    if (!self.proposer_chain.contains_key(&vote)) {
                        self.orphan_buffer.entry(vote).or_insert(Vec::new()).push(block);
                        // self.orphan_buffer.insert(vote, block);
                        return true;
                    }
                }
                return false;
            }
        }
    }

    pub fn insert(&mut self, block: &Block) -> InsertStatus {

        if is_orphan(block) {
            return InsertStatus::Orphan;
        }
        
        // All references inside the block are guaranteed to be present
        let block_hash = block.hash();

        match block.content {
            Content::Proposer(content) => {
                
                // Add self hash and remove referenced proposer hashes from `unref_proposers`
                self.unref_proposers.push(block_hash);
                for ref_proposer in content.proposer_refs {
                    let result = self.unref_proposers.iter().position(|x| *x == ref_proposer);
                    match result {
                        Some(index) => self.unref_proposers.remove(index),
                        None => println!("How come you trying to reference something not in `unref_proposers`?"),
                    }
                }

                let parent_meta = self.proposer_chain[&block.header.parenthash];
                let block_level = parent_meta.level + 1;
                // Add to `level2proposer` if first proposer at its level
                if !self.level2proposer.contains_key(&block_level) {
                    self.level2proposer.insert(block_level, block_hash);
                }
                // Add to `level2allproposers`
                self.level2allproposers.entry(block_level).or_insert(Vec::new()).push(block_hash);

                // Add to `proposer_chain` and update tip
                let metablock = Metablock {
                    block: *block,
                    level: block_level,
                };
                self.proposer_chain.insert(block_hash, metablock);
                if metablock.level > self.proposer_depth {
                    self.proposer_depth = metablock.level;
                    self.proposer_tip = block_hash;
                }
            }

            Content::Voter(content) => {
                let chain_num = content.chain_num;

                // BEHOLD
                // The below code is inaccurate: votes aren't counted from every block, 
                // only the blocks belonging to the longest chain. So this is a major TODO.
                // Bhavana will work on this 4/25. 

                // go through all votes, update proposer2votecount and chain2level
                let mut max_vote_level: u32 = self.chain2level[&chain_num];
                for vote in content.votes {
                    // update proposer2votecount
                    let counter = self.proposer2votecount.entry(vote).or_insert(0);
                    *counter += 1;
                    // update max vote level variable
                    let block_level = self.proposer_chain[&vote].level;
                    let max_vote_level = cmp::max(max_vote_level, block_level);
                }
                self.chain2level.insert(&chain_num, max_vote_level);

                // add to voter chain and update tip
                let parent_meta = self.voter_chains[(chain_num-1) as usize][&block.header.parenthash];
                let metablock = Metablock {
                    block: *block,
                    level: parent_meta.level + 1
                };
                self.voter_chains[(chain_num-1) as usize].insert(block_hash, metablock);
                if metablock.level > self.voter_depths[(chain_num-1) as usize] {
                    self.voter_depths[(chain_num-1) as usize] = metablock.level;
                    self.voter_tips[(chain_num-1) as usize] = block_hash;
                }
            }
        }

        let result = self.orphan_buffer.remove(&block_hash);
        match result {
            Some(orphan_blocks) => {
                let count: u32 = 0;
                for orphan_block in orphan_blocks {
                    let status = self.insert(orphan_block);
                    match status {
                        InsertStatus::Valid => count += 1,
                        InsertStatus::Orphan => {},
                    }
                }
                println!("{:?} unorphaned {} blocks, out of {} waiting on it", block_hash, count, orphan_blocks.len());
            },
            None => println!("No orphan blocks waiting on {:?}", block_hash),
        }
    }

    pub fn get_proposer_tip(&self) -> H256 {
        self.proposer_tip
    }

    pub fn get_voter_tip(&self, chain_num: u32) -> H256 {
        self.voter_tips[chain_num as usize]
    }

}

// write tests for blockchain
#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    // use crate::block::test::generate_random_block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn blockchain_init() {
        // 10 voting chains
        let mut blockchain = Blockchain::new(10);
    }
}
