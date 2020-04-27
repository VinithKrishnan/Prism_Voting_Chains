use crate::crypto::hash::H256;
use crate::blockchain::Blockchain;

use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

use log::debug;

//state required by ledger-manager
pub struct LedgerManagerState {
    pub last_level_processed: u32,
    pub leader_sequence: Vec<H256>,
    pub proposer_blocks_processed: HashSet<H256>,
    pub tx_confirmed: HashSet<H256>,
}

//ledger-manager which will periodically loop and confirm the transactions 
pub struct LedgerManager {
    pub ledger_manager_state: LedgerManagerState,
    pub blockchain: Arc<Mutex<Blockchain>>,
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
    pub fn ledger_manager_loop(&mut self) {
        loop{
            //Step 1
            let leader_sequence = self.get_leader_sequence();
            
            //Step 2
            //let tx_sequence = self.get_transaction_sequence(&leader_sequence);
            
            //Step 3
            //self.confirm_transactions(&tx_sequence);
            
            thread::sleep(Duration::from_secs(1));
        }
    }

    fn get_leader_sequence(&mut self) -> Vec<H256> {
        let locked_blockchain = self.blockchain.lock().unwrap();
        
        let mut leader_sequence: Vec<H256> = vec![];

        //TODO: This is a workaround for now till we have some DS which asserts that
        //all voter chains at a particular level has voted
        let level_start = self.ledger_manager_state.last_level_processed + 1;
        let level_end = locked_blockchain.proposer_depth + 1;
        for level in level_start..level_end {
            let proposers = locked_blockchain.level2allproposers[&level];
            
            let mut max_vote_count = 0;
            let mut leader: H256 = [0; 32].into();
            //Assumption: if a vote for proposer not present, assumed to be 0
            //When above TODO is done, then this will not be needed or modified accordingly
            for proposer in &proposers {
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

            
            debug!("Adding leader at level {}. Leader hash: {:?}", level, leader);
            leader_sequence.push(leader);
            self.ledger_manager_state.last_level_processed = level;

        }

        leader_sequence
    }
    

    fn get_transaction_sequence(&mut self) {

    }

    fn confirm_transactions(& self) {

    }
}