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
use multimap::MultiMap;


pub struct Metablock {
    pub block: Block,
    pub level: u32,
}

// TODO: check if u32 is required to record depth

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
    pub chain2level: HashMap<u32, u32>,

    // orphan buffer stores a mapping between missing reference and block
    // use multimap as many blocks could wait on a single reference.
    pub orphan_buffer: MultiMap<H256, Block>,
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
        }
        proposer_chain.insert(proposer_hash, metablock);
        let proposer_tip = proposer_hash;

        let mut voter_chains = Vec::new();
        let mut voter_tips = Vec::new();
        let mut voter_depths = Vec::new();
        let mut chain2level = HashMap::new();
        for chain_num in 1..m {
            let mut tmp_chain = HashMap::new();
            let voter = genesis_voter(i);
            let voter_hash = voter.hash();
            let metablock = Metablock {
                block: voter,
                level: 1,
            }
            tmp_chain.insert(voter_hash, metablock);
            // might have to make a copy of tmp_chain?
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

            orphan_buffer: MultiMap::new(),
        }
    }

    pub fn insert(&mut self, block: &Block) {
        // IMP TODO: If block has missing refs, add to orphan buffermap
        
        // Assume all block references are present
        // TODO: Haven't added all checks -- parent present then unwrap with confidence etc
        match block.content {
            Content::Proposer(c) => {
                // Check all references
                if (!self.proposer_chain.contains_key(block.header.parenthash)) {
                    // parent proposer not found, add to orphan buffer
                    self.orphan_buffer.insert(block.header.parenthash, block);
                    continue;
                }

                let mut orphan: bool = false;
                for ref_proposer in block.content.proposer_refs {
                    if (!self.proposer_chain.contains_key(ref_proposer)) {
                        let orphan = true;
                        self.orphan_buffer.insert(ref_proposer, block);
                        break;
                    }
                }
                if (orphan) {
                    continue;
                }

                // At this point, all references are guaranteed to be present

                // add selfhash to unreferenced, remove referenced proposers in content
                let block_hash = block.hash();
                self.unref_proposers.push(block_hash);
                for ref_proposer in block.content.proposer_refs {
                    // safe removal from self.unref_proposers vec
                    let result = self.unref_proposers.iter().position(|x| *x == ref_proposer);
                    match result {
                        Some(index) => self.unref_proposers.remove(index);
                    }
                }
                // unwrap is safe since all references are present
                // let parent_meta = self.proposer_chain.get(&block.header.parenthash).unwrap();
                let parent_meta = self.proposer_chain[block.header.parenthash];
                let metablock = Metablock {
                    block: *block,
                    level: parent_meta.level + 1,
                }

                // if this is the first proposer block at its level, update level2proposer map
                if !self.level2proposer.contains_key(&metablock.level) {
                    self.level2proposer.insert(metablock.level, block_hash);
                }
                // add the proposer block hash to the list of its level
                self.level2allproposers.entry(metablock.level).or_insert(Vec::new()).push(block_hash);

                // add to proposer chain and update proposer tip if depth has increased
                self.proposer_chain.insert(block_hash, metablock);
                if metablock.level > self.proposer_depth {
                    self.proposer_depth = metablock.level;
                    self.proposer_tip = block_hash,
                }

                // IMP TODO: check if any orphaned blocks can be unorphaned
                // This is going to cause some major changes are orphan handling can have cascading effects.
            }

            Content::Voter(c) => {
                let chain_num = *block.content.chain_num;

                // Check if all references are present
                if (!self.voter_chains[chain_num-1].contains_key(block.header.parenthash)) {
                    // parent proposer not found, add to orphan buffer
                    self.orphan_buffer.insert(block.header.parenthash, block);
                    continue;
                }

                let mut orphan: bool = false;
                for vote in block.content.votes {
                    if (!self.proposer_chain.contains_key(vote)) {
                        let orphan = true;
                        self.orphan_buffer.insert(vote, block);
                        break;
                    }
                }
                if (orphan) {
                    continue;
                }

                // At this point, all references are guaranteed to be present

                // go through all votes, update proposer2votecount and chain2level
                let mut max_vote_level: u32 = self.chain2level[chain_num];
                for vote in block.content.votes {
                    // update proposer2votecount
                    let counter = self.proposer2votecount.entry(vote).or_insert(0);
                    *counter += 1;
                    // update max vote level variable
                    let block_level = self.proposer_chain[&vote].level;
                    let max_vote_level = max(max_vote_level, block_level);
                }
                self.chain2level.insert(chain_num, max_vote_level);

                // add to voter chain and update tip if required
                // let parent_meta = self.voter_chains[chain_num-1].get(&block.header.parenthash).unwrap();
                let parent_meta = self.voter_chains[chain_num-1][block.header.parenthash];
                let metablock = Metablock {
                    block: *block,
                    level: parent_meta.level + 1
                }
                self.voter_chains[chain_num-1].insert(block_hash, metablock);
                if metablock.level > self.voter_depths[chain_num-1] {
                    self.voter_depths[chain_num-1] = metablock.level;
                    self.voter_tips[chain_num-1] = block_hash,
                }

                // IMP TODO: check if any orphaned blocks can be unorphaned
                // This is going to cause some major changes are orphan handling can have cascading effects.
            }
        }
    }

    pub fn get_proposer_tip(&self) -> H256 {
        self.proposer_tip
    }

    pub fn get_voter_tip(&self, chain_num) -> H256 {
        self.voter_tips[chain_num]
    }

}

// write tests for blockchain