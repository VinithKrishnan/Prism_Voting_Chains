use crate::transaction::{self, UtxoInput, UtxoOutput, SignedTransaction};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::address;

use std::collections::HashMap;

use log::debug;

#[derive(Debug, Default, Clone)]
pub struct UtxoState{
    pub state_map: HashMap<UtxoInput, UtxoOutput>,  
}

impl  UtxoState {
    //TODO: Should call ico()
    pub fn new() -> Self {
        UtxoState{
            state_map: HashMap::new(),
        }
    }
    
    //TODO: Should take Vec<SignedTransaction> for more general purpose
    //As we will be giving only one tx at a time, for now it is fine
    pub fn update_state(&mut self, signed_tx: &SignedTransaction) {
        for tx_input in &signed_tx.tx.tx_input {
            self.state_map.remove(tx_input);
        }
        
        for (i, tx_output) in (&signed_tx.tx.tx_output).iter().enumerate() {
            let tx_input = UtxoInput{tx_hash: signed_tx.tx.hash(), idx: i as u8};
            self.state_map.insert(tx_input, tx_output.clone());
        }
    }

    //Should it be a "function" rather than "method" of UtxoState??
    //1. Signature check
    //2. Owner match
    //3. Double Spend
    //4. Input/Output total match
    pub fn is_tx_valid(&self, signed_tx: &SignedTransaction) -> bool {
        debug!("current signed_tx {:?}", signed_tx);

        if !transaction::verify(&signed_tx.tx, &signed_tx.signature, &signed_tx.public_key){
            debug!("tx didn't pass signature check!");
            return false;
        }
        
        let owner_address = address::address_from_public_key_vec_ref(&signed_tx.public_key);
        let mut total_input_value = 0;
        for input in &signed_tx.tx.tx_input {
            if !self.state_map.contains_key(&input) {
               debug!("Input is {:?}",input);
               debug!("tx is double spend as input is not there in State!");
               return false;  
            }

            let output = &self.state_map[&input];
            if output.receipient_addr != owner_address {
               debug!("owner of tx input doesn't match to previous tx output");
               debug!("input addreess {:?}", owner_address);
               debug!("output address {:?}", output.receipient_addr);
               return false;
            }
            total_input_value = output.value;
        }
        
        let mut total_output_value = 0;
        for output in &signed_tx.tx.tx_output {
             total_output_value += output.value;
        }
  
        if total_input_value != total_output_value {
           debug!("Input sum didn't match to output sum for tx");
           return false;
        }

        true
    }
}
