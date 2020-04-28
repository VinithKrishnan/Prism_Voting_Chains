
use crate::network::server::Handle as ServerHandle;
use crate::block::{self, *};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::merkle::MerkleTree;
use crate::blockchain::{Blockchain, InsertStatus};

use log::info;
use bigint::uint::U256;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time;

use std::thread;

const TOTAL_SORTITION_WIDTH: u64 = std::u64::MAX;
pub const PROPOSER_INDEX: u32 = 0;
pub const FIRST_VOTER_IDX: u32 = 1;


pub enum BlockResult {
    Pass,
    Fail,
}

//same as in miner
pub fn sortition_hash(hash: &H256, difficulty: &H256, num_voter_chains: u32) -> Option<u32> {
    let hash = U256::from_big_endian(hash.as_ref());
    let difficulty = U256::from_big_endian(difficulty.as_ref());
    let multiplier = difficulty / TOTAL_SORTITION_WIDTH.into();
    //let precise: f32 = (1 / f32::from(num_voter_chains + 1)) * TOTAL_SORTITION_WIDTH as f32;
    let precise: f32 = (1.0 / (num_voter_chains + 1) as f32) * TOTAL_SORTITION_WIDTH as f32;
    let proposer_sortition_width: u64 = precise.ceil() as u64
    let proposer_width = multiplier * proposer_sortition_width.into();
    if hash < proposer_width {
        Some(PROPOSER_INDEX)
    } else if hash < difficulty {
        let voter_idx = (hash - proposer_width) % num_voter_chains;
        Some(FIRST_VOTER_IDX + voter_idx)
    } else {
        println!("Why you sortitioning something that is not less than difficulty?");
        None
    }
}
//PoW and sortition id
pub fn check_pow_sortition_id(block: &Block, blockchain: &Blockchain) -> BlockResult {
    //no need to lock blockchain here since we passed locked blcokchain
    let sortition_id = sortition_hash(&block.hash(), &block.header.difficulty,&blockchain.num_voter_chains);
    if sortition_id.is_none() {
        return BlockResult::Fail;
    }
    let correct_sortition_id = match &block.content {
        Content::Proposer(_) => PROPOSER_INDEX,
        Content::Voter(content) => content.chain_num + FIRST_VOTER_IDX,
    };
    if sortition_id != correct_sortition_id {
        return BlockResult::Fail;
    }
    return BlockResult::Pass;
}

//check merkle tree there
pub fn check_sortition_proof(block: &Block, blockchain: &Blockchain) -> BlockResult {
    let sortition_id = sortition_hash(&block.hash(), &block.header.difficulty,&blockchain.num_voter_chains);
    if sortition_id.is_none() {
        return BlockResult::Fail;
    }
    if !verify(
        &block.header.merkle_root,
        &block.content.hash(),
        &block.sortition_proof,
        sortition_id as usize,
        (blockchain.num_voter_chains + FIRST_VOTER_INDEX) as usize,
    ) {
        return BlockResult::Fail;
    }
    return BlockResult::Pass;
}
