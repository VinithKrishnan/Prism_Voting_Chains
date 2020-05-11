use crate::crypto::hash::{H256, Hashable};
use crate::blockchain::Blockchain;
use crate::block::Content;
use crate::transaction::SignedTransaction;
use crate::utxo::UtxoState;

use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use statrs::distribution::{Discrete, Poisson, Univariate};

use log::debug;

//state required by ledger-manager
pub struct LedgerManagerState {
    pub last_level_processed: u32,
    pub leader_sequence: Vec<H256>,
    pub proposer_blocks_processed: HashSet<H256>,
    pub tx_confirmed: HashSet<H256>,
}

//ledger-manager will periodically loop and confirm the transactions 
pub struct LedgerManager {
    pub ledger_manager_state: LedgerManagerState,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub utxo_state: UtxoState,
}

impl LedgerManager {
    pub fn new(blockchain: &Arc<Mutex<Blockchain>>) -> Self {
        let ledger_manager_state = LedgerManagerState{
            last_level_processed: 0,
            proposer_blocks_processed: HashSet::new(),
            leader_sequence: Vec::new(),
            tx_confirmed: HashSet::new(),
        };

        LedgerManager {
            ledger_manager_state: ledger_manager_state,
            blockchain: Arc::clone(blockchain),
            utxo_state: UtxoState::new(),
        }
    }

    pub fn start(mut self) {
        thread::Builder::new()
        .name("ledger_manager".to_string())
        .spawn(move || {
            self.ledger_manager_loop();
        })
        .unwrap();
    }

    //Three Steps
    //1. Get the leader sequence
    //2. Get Transaction sequence
    //3. Sanitize Tx and update UTXO state
    //All 3 steps are done in the loop
    //
    fn ledger_manager_loop(&mut self) {
        loop{
            //Step 1
            //let leader_sequence = self.get_leader_sequence();
            
            //This one uses the algorithm described in Prism Paper
            let leader_sequence = self.get_confirmed_leader_sequence();
            
            //Step 2
            let tx_sequence = self.get_transaction_sequence(&leader_sequence);
            
            //Step 3
            self.confirm_transactions(&tx_sequence);
            
            thread::sleep(Duration::from_secs(1));
        }
    }

    fn get_leader_sequence(&mut self) -> Vec<H256> {
        let locked_blockchain = self.blockchain.lock().unwrap();
        
        let mut leader_sequence: Vec<H256> = vec![];

        //TODO: This is a workaround for now till we have some DS which asserts that
        //all voter chains at a particular level has voted
        // level2votes: how many votes have been casted at level i
        let level_start = self.ledger_manager_state.last_level_processed + 1;
        let level_end = locked_blockchain.proposer_depth + 1;
        for level in level_start..level_end {
            let proposers = &locked_blockchain.level2allproposers[&level];
            
            let mut max_vote_count = 0;
            let mut leader: H256 = [0; 32].into();
            //Assumption: if a vote for proposer not present, assumed to be 0
            //When above TODO is done, then this will not be needed or modified accordingly
            for proposer in proposers {
                if locked_blockchain.proposer2votecount.contains_key(proposer) {
                    let vote_count = locked_blockchain.proposer2votecount[proposer];
                    if vote_count > max_vote_count {
                        max_vote_count = vote_count;
                        leader = *proposer;
                    }
                }
            }
            
            //break out as there is no point going forward as no leader found at this level
            if max_vote_count == 0 {
                break;
            }

            debug!("Adding leader at level {}, leader hash: {:?}, max votes: {}", level, leader, max_vote_count);
            leader_sequence.push(leader);
            self.ledger_manager_state.last_level_processed = level;
        }

        leader_sequence
    }
    
    fn get_confirmed_leader_sequence(&mut self) -> Vec<H256> {
        let mut leader_sequence: Vec<H256> = vec![];

        //Locking Blockchain to get proposer_depth currently. Then dropping the lock
        //Will be holding locj for each level processing inside the subroutine
        let locked_blockchain = self.blockchain.lock().unwrap();

        let level_start = self.ledger_manager_state.last_level_processed + 1;
        let level_end = locked_blockchain.proposer_depth + 1;

        drop(locked_blockchain);

        for level in level_start..level_end {
            let leader: Option<H256> = self.confirm_leader(level);

            match leader {
                Some(leader_hash) => {  
                    debug!("Adding leader at level {}, leader hash: {:?}", level, leader_hash);
                    leader_sequence.push(leader_hash);
                    self.ledger_manager_state.last_level_processed = level;
                }

                None => {
                    debug!("Unable to confirm leader at level {}", level);
                    debug!("Returning from get_confirmed_leader_sequence func");
                    break; // TODO: Will this break out of loop??
                }
            }
        }

        leader_sequence     
    }

    //we use the confirmation policy from https://arxiv.org/abs/1810.08092
    //This function is heavily borrowed from implementation provided in the actual Prism codebase
    //https://github.com/yangl1996/prism-rust/
    fn confirm_leader(&mut self, level: u32) -> Option<H256> {

        //TODO: Have to define globally beta and quartile somewhere
        let beta = 0.1;
        let quantile = 0.0001;

        let locked_blockchain = self.blockchain.lock().unwrap();
        
        let proposer_blocks = &locked_blockchain.level2allproposers[&level];
        let mut new_leader: Option<H256> = None;

        // collect the depth of each vote on each proposer block
        //// chain number and vote depth casted on the proposer block
        let mut votes_depth: HashMap<&H256, Vec<u32>> = HashMap::new(); 

        // collect the total votes on all proposer blocks, and the number of
        // voter blocks mined after those votes are casted
        let mut total_vote_count: u32 = 0;
        let mut total_vote_blocks: u32 = 0;

        for block in proposer_blocks {
            if locked_blockchain.proposer2voterinfo.contains_key(block) {
                
                //TODO: We might also need number of voter blocks at a particular level of a voter chain
                //This is not urgent as we can **assume**, there is one block at each level
                let voters_info = &locked_blockchain.proposer2voterinfo[block];
                let mut vote_depth: Vec<u32> = vec![];
                for (voter_chain, voter_block) in voters_info {

                    let voter_block_level = locked_blockchain.voter_chains[*voter_chain as usize][voter_block].level;
                    let voter_chain_level = locked_blockchain.voter_depths[*voter_chain as usize];
                    
                    total_vote_blocks += 1;
                    let this_vote_depth = voter_chain_level - voter_block_level + 1;

                    //TODO: As no voter forking for now, this will be equal to depth
                    total_vote_count += this_vote_depth;
                    vote_depth.push(this_vote_depth);
                }
                votes_depth.insert(block, vote_depth);
            }
        }

        // no point in going further if less than 3/5 votes are cast
        if total_vote_count > locked_blockchain.num_voter_chains * 3 / 5  {
            // calculate the average number of voter blocks mined after
            // a vote is casted. we use this as an estimator of honest mining
            // rate, and then derive the believed malicious mining rate
            let avg_vote_blocks = total_vote_blocks as f64 / f64::from(total_vote_count);
            
            // expected voter depth of an adversary
            let adversary_expected_vote_depth = avg_vote_blocks / (1.0 - beta) * beta;
            let poisson = Poisson::new(f64::from(adversary_expected_vote_depth)).unwrap();

            // for each block calculate the lower bound on the number of votes
            let mut votes_lcb: HashMap<&H256, f64> = HashMap::new();
            let mut total_votes_lcb: f64 = 0.0;
            let mut max_vote_lcb: f64 = 0.0;

            for block in proposer_blocks {
                let votes = votes_depth.get(block).unwrap();

                let mut block_votes_mean: f64 = 0.0; // mean E[X]
                let mut block_votes_variance: f64 = 0.0; // Var[X]
                let mut block_votes_lcb: f64 = 0.0;
                for depth in votes.iter() {
                    // probability that the adversary will remove this vote
                    let mut p: f64 = 1.0 - poisson.cdf((*depth as f64 + 1.0).into()) as f64;
                    for k in 0..(*depth as u64) {
                        // probability that the adversary has mined k blocks
                        let p1 = poisson.pmf(k) as f64;
                        // probability that the adversary will overtake 'depth-k' blocks
                        let p2 = (beta / (1.0 - beta))
                            .powi((depth - k as u32 + 1) as i32);
                        p += p1 * p2;
                    }
                    block_votes_mean += 1.0 - p;
                    block_votes_variance += p * (1.0 - p);
                }

                // using gaussian approximation
                let tmp = block_votes_mean - (block_votes_variance).sqrt() * quantile;
                if tmp > 0.0 {
                    block_votes_lcb += tmp;
                }
                votes_lcb.insert(block, block_votes_lcb);
                total_votes_lcb += block_votes_lcb;

                if max_vote_lcb < block_votes_lcb {
                    max_vote_lcb = block_votes_lcb;
                    new_leader = Some(*block);
                }
                // In case of a tie, choose block with lower hash.
                if (max_vote_lcb - block_votes_lcb).abs() < std::f64::EPSILON
                    && new_leader.is_some()
                {
                    // TODO: is_some required?
                    if *block < new_leader.unwrap() {
                        new_leader = Some(*block);
                    }
                }
            }
            // check if the lcb_vote of new_leader is bigger than second best ucb votes
            let remaining_votes = f64::from(locked_blockchain.num_voter_chains) - total_votes_lcb;

            // if max_vote_lcb is lesser than the remaining_votes, then a private block could
            // get the remaining votes and become the leader block
            if max_vote_lcb <= remaining_votes || new_leader.is_none() {
                new_leader = None;
            } else {
                for p_block in proposer_blocks{
                    // if the below condition is true, then final votes on p_block could overtake new_leader
                    if max_vote_lcb < votes_lcb.get(p_block).unwrap() + remaining_votes
                        && *p_block != new_leader.unwrap()
                    {
                        new_leader = None;
                        break;
                    }
                    //In case of a tie, choose block with lower hash.
                    if (max_vote_lcb - (votes_lcb.get(p_block).unwrap() + remaining_votes)).abs()
                        < std::f64::EPSILON
                        && *p_block < new_leader.unwrap()
                    {
                        new_leader = None;
                        break;
                    }
                }
            }
        }

        new_leader
    }

    // needs to process parent as well
    fn get_transaction_sequence(&mut self, leader_sequence: &Vec<H256>) -> Vec<SignedTransaction> {        
        let locked_blockchain = self.blockchain.lock().unwrap();

        let mut tx_sequence: Vec<SignedTransaction> = Vec::new();

        //TODO: Should we do it recusrively? Like should we also see references to
        //proposer references of leader?
        //TODO: Also we should refactor it later
        for leader in leader_sequence {
            let leader_block = &locked_blockchain.proposer_chain[leader].block;

            //processing parent and proposer refs
            let mut proposer_refs_to_process: Vec<H256> = Vec::new();
            let mut leader_txs: Vec<SignedTransaction> = Vec::new();
            match &leader_block.content {
                Content::Proposer(content) => {
                    //parent and proposer_refs of leader
                    let parent = &content.parent_hash;
                    let proposer_refs = &content.proposer_refs;
                    
                    if !self.ledger_manager_state.proposer_blocks_processed.contains(parent) {
                        proposer_refs_to_process.push(*parent);
                    }

                    for proposer_ref in proposer_refs {
                        if !self.ledger_manager_state.proposer_blocks_processed.contains(proposer_ref) {
                            proposer_refs_to_process.push(*proposer_ref);
                        }
                    }

                    //txs of leader
                    leader_txs = content.transactions.clone(); 
                }
                _ => {

                }
            }

            //TODO: Do we have to do match in this and previous loop as we know it will always
            //match to Proposer(content). Can we unwrap??
            for proposer_ref in &proposer_refs_to_process {
                let proposer_block = &locked_blockchain.proposer_chain[proposer_ref].block;
                match &proposer_block.content {
                    Content::Proposer(content) => {
                        tx_sequence.append(&mut content.transactions.clone()); 
                    }
                    _ => {

                    }
                }             
                
                self.ledger_manager_state.proposer_blocks_processed.insert(*proposer_ref);
            }

            //appending leader txs finally
            //adding leader to proposer_blocks_processed
            tx_sequence.append(&mut leader_txs);
            self.ledger_manager_state.proposer_blocks_processed.insert(*leader);
        }

        tx_sequence
    }

    fn confirm_transactions(&mut self, tx_sequence: &Vec<SignedTransaction>) {
        for tx in tx_sequence {
            //if already processed continue
            if self.ledger_manager_state.tx_confirmed.contains(&tx.hash()) {
                continue;
            }

            //check for validity
            //if valid, update utxo_state and add to confirmed transactions
            if self.utxo_state.is_tx_valid(tx){
                self.utxo_state.update_state(tx);
                self.ledger_manager_state.tx_confirmed.insert(tx.hash());
            }
        }
    }
}
