use crate::crypto::hash::H256;

use std::collections::HashSet;

//state required by ledger-manager
pub struct LedgerManagerState {
    pub last_level_processed: u32,
    pub proposer_blocks_processed: HashSet<H256>,
    pub leader_sequence: Vec<H256>,
    pub tx_confirmed: HashSet<H56>,
}

//ledger-manager which will periodically loop and confirm the transactions 
pub struct LedgerManager {
    pub ledger_manager_state: LedgerManagerState,
}

impl LedgerManager {
    pub fn new() -> Self {
        let ledger_manager_state = LedgerManagerState{
            last_level_processed: 0,
            proposer_blocks_processed: HashSet::new(),
            leader_sequence: Vec::new(),
            tx_confirmed: HashSet::new(),
        }

        LedgerManager {
            ledger_manager_state: ledger_manager_state
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

    pub fn ledger_manager_loop(&mut self) {

    }
}