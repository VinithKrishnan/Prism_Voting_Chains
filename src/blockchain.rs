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


pub struct Metablock {
    pub block: Block,
    pub height: u32,
}

pub struct Blockchain {
    pub proposer_chain: HashMap<H256, Metablock>,
    pub proposer_tip: H256,
    pub proposer_depth: u32,

    pub voter_chains: Vec<HashMap<H256, Metablock>>,
    pub voter_tips: Vec<H256>,
    pub voter_depths: Vec<u32>,
    // list of unreferenced proposer blocks
    pub unref_proposers: Vec<H256>,
    // Hash of first proposer block seen corresponding to each level
    pub level2proposer: HashMap<u32, H256>,
    // Last voted level corresponding to each voter chain 
    pub chain2level: HashMap<u32, u32>

    // TODO: add orphan buffer for proposer and voter chains 
}

impl Blockchain {
    pub fn new(m: u32) -> Self {
        // genesis for proposer and voter chains
        let mut proposer_chain = HashMap::new();
        let proposer = genesis_proposer();
        let proposer_hash = proposer.hash();
        let metablock = Metablock {
            block: proposer,
            height: 1,
        }
        proposer_chain.insert(proposer_hash, metablock);
        let proposer_tip = proposer_hash;

        let mut voter_chains = Vec::new();
        let mut voter_tips = Vec::new();
        let mut chain2level = HashMap::new();
        let mut voter_depths = Vec::new();
        for i in 1..m {
            let mut tmp_chain = HashMap::new();
            voter = genesis_voter(i);
            voter_hash = voter.hash();
            let metablock = Metablock {
                block: voter,
                height: 1,
            }
            tmp_chain.insert(voter_hash, metablock);
            voter_chains.push(tmp_chain);
            voter_tips.push(voter_hash);
            chain2level.insert(i, 0);
            voter_depths.push(1);
        } 

        let mut unref_proposers = Vec::new();
        unref_proposers.push(proposer_hash);

        let mut level2proposer = HashMap::new();
        level2proposer.insert(1, proposer_hash);

        Blockchain {
            proposer_chain: proposer_chain,
            proposer_tip: proposer_hash,
            proposer_depth: 1,
            voter_chains: voter_chains,
            voter_tips: voter_tips,
            voter_depths: voter_depths,
            unref_proposers: unref_proposers,
            level2proposer: level2proposer,
            chain2level: chain2level,
        }
    }

    pub fn insert(&mut self, block: &Block) {
        // TODO: If block has missing refs, add to orphan buffermap
        
        // Haven't added all checks -- parent present then unwrap with confidence etc
        match block.content {
            Content::Proposer(c) => {
                // add selfhash to unreferenced, remove referenced proposers in content
                let block_hash = block.hash();
                self.unref_proposers.push(block_hash);
                for ref_proposer in block.content.proposer_refs {
                    // remove from self.unref_proposers
                    // TODO check what happens if ref_proposer not in self.unref_proposer. shouldn't panic
                    let index = self.unref_proposers.iter().position(|x| *x == ref_proposer).unwrap();
                    self.unref_proposers.remove(index);
                }
                
                let parent_meta = self.proposer_chain.get(&block.header.parenthash).unwrap();
                let metablock = Metablock {
                    block: block,
                    height: parent_meta.height + 1
                }

                // if this is the first proposer block at its level, update level2proposer map
                if (!self.level2proposer.contains_key(&metablock.height)) {
                    self.level2proposer.insert(metablock.height, block_hash);
                }

                // add to proposer chain and update proposer tip if depth has increased
                self.proposer_chain.insert(block_hash, metablock);
                if metablock.height > self.proposer_depth {
                    self.proposer_depth = metablock.height;
                    self.proposer_tip = block_hash,
                }
            }

            Content::Voter(c) => {
                let chain_num = block.content.chain_num;
                // calculate max level voted
                let mut max_vote_level: u32 = self.chain2level[chain_num];
                for vote in block.content.votes {
                    // unwrap confidence?
                    let block_level = self.proposer_chain.get(&vote).unwrap().height;
                    let max_vote_level = max(max_vote_level, block_level);
                }
                self.chain2level.insert(chain_num, max_vote_level);

                // add to voter chain and update tip if required
                let parent_meta = self.voter_chains[chain_num-1].get(&block.header.parenthash).unwrap();
                let metablock = Metablock {
                    block: block,
                    height: parent_meta.height + 1
                }
                self.voter_chains[chain_num-1].insert(block_hash, metablock);
                if metablock.height > self.voter_depths[chain_num-1] {
                    self.voter_depths[chain_num-1] = metablock.height;
                    self.voter_tips[chain_num-1] = block_hash,
                }
            }
        }
    }

    pub fn get_proposer_tip(&self) {
        self.proposer_tip
    }

    pub fn get_voter_tip(&self, chain_num) {
        self.voter_tips[chain_num]
    }

}