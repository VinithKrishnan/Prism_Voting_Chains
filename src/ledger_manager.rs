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