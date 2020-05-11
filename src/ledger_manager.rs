use crate::crypto::hash::{H256, Hashable};
use crate::blockchain::Blockchain;
use crate::block::Content;
use crate::transaction::SignedTransaction;
use crate::utxo::UtxoState;

use std::collections::HashSet;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};

use std::sync::{Arc, Mutex};

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
    pub utxo_state: Arc<Mutex<UtxoState>>,
}

impl LedgerManager {
    pub fn new(blockchain: &Arc<Mutex<Blockchain>>, utxo_state: &Arc<Mutex<UtxoState>>) -> Self {
        let ledger_manager_state = LedgerManagerState{
            last_level_processed: 0,
            proposer_blocks_processed: HashSet::new(),
            leader_sequence: Vec::new(),
            tx_confirmed: HashSet::new(),
        };

        LedgerManager {
            ledger_manager_state: ledger_manager_state,
            blockchain: Arc::clone(blockchain),
            utxo_state: Arc::clone(utxo_state),
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
            let leader_sequence = self.get_leader_sequence();
            
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

            println!("Adding leader at level {}, leader hash: {:?}, max votes: {}", level, leader, max_vote_count);
            leader_sequence.push(leader);
            self.ledger_manager_state.last_level_processed = level;
        }

        leader_sequence
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
            
            let mut proposer_refs_to_process: Vec<H256> = Vec::new();
            let mut leader_txs: Vec<SignedTransaction> = Vec::new();
            match &leader_block.content {
                Content::Proposer(content) => {
                    //proposer_refs of leader
                    let proposer_refs = &content.proposer_refs;
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
        let mut locked_utxostate = self.utxo_state.lock().unwrap();
        for tx in tx_sequence {
            println!("processing {:?}", tx.hash());
            //if already processed continue
            if self.ledger_manager_state.tx_confirmed.contains(&tx.hash()) {
                println!("{:?} already confirmed", tx.hash());
                continue;
            }

            //check for validity
            //if valid, update utxo_state and add to confirmed transactions
            if locked_utxostate.is_tx_valid(tx){
                println!("{:?} valid", tx.hash());
                locked_utxostate.update_state(tx);
                self.ledger_manager_state.tx_confirmed.insert(tx.hash());
                println!("Confirmed trans hash {} at {}", tx.hash(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros());
                // Print UTXO state
                locked_utxostate.print();
            } else {
                println!("{:?} invalid", tx.hash());
            }
        }
        drop(locked_utxostate);
    }
}
