use crate::crypto::hash::H256;
use crate::transaction::{SignedTransaction,UtxoInput};
use crate::crypto::hash::Hashable;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::convert::TryInto;
  
#[derive(Debug)]
pub struct TransactionMempool{
    
    //counter for storage_index for btree
    counter: u32,

    //tx_hash to TxStore 
    hash_to_txstore:HashMap<H256,TxStore>,
    //map from tx_input to tx_hash (required for db check and  dependant tx removal)
    input_to_hash:HashMap<UtxoInput,H256>,
    // storage_index to txhash, used for maintaining FIFO order
    index_to_hash:BTreeMap<u32,H256>,



}
#[derive(Debug, Clone)]
pub struct TxStore{  //used for storing a tx and its btree index

    pub signed_tx: SignedTransaction,
    
    //storage index for btree
    index: u32,

}
  
impl TransactionMempool{
    pub fn new() -> Self{
      TransactionMempool{ counter: 0,
      hash_to_txstore: HashMap::new(),
      input_to_hash: HashMap::new(),
      index_to_hash: BTreeMap::new(), 
      }
    }

    pub fn insert(&mut self,tx:SignedTransaction) {
            
            let hash = tx.hash();

            let txstore = TxStore{
                signed_tx:tx,
                index:self.counter,
            };
            self.counter +=1;
            //QUESTION:Should I perform signature validation before inserting?
            for input in &txstore.signed_tx.tx.tx_input {
                self.input_to_hash.insert(input.clone(), hash);
            }
            
            self.index_to_hash.insert(txstore.index,hash);

            self.hash_to_txstore.insert(hash,txstore);

    }

    // https://doc.rust-lang.org/std/option/
    // https://doc.rust-lang.org/edition-guide/rust-2018/error-handling-and-panics/the-question-mark-operator-for-easier-error-handling.html
    // ^ handy constructs for error handling

    pub fn get(&self, h: &H256) -> Option<&TxStore> {
        let txstore = self.hash_to_txstore.get(h)?;
        Some(txstore)
    }

    // checks whether tx is already in mempool
    //called before insert is called
    // Could be part of insert method itself
    pub fn contains(&self, h: &H256) -> bool {
        self.hash_to_txstore.contains_key(h)
    }

    //checks whether new txs input had already been used by tx in mempool
    //called before insert is called
    // Could be part of insert method itself
    pub fn is_double_spend(&self, inputs: &[UtxoInput]) -> bool {
        for input in inputs.iter(){
            if self.input_to_hash.contains_key(input) {
                return true
            }
        }
        false
    }

    // removes a tx from mempool and returns the tx, used by delete_dependent_txs
    pub fn delete_and_get(&mut self, hash: &H256) -> Option<TxStore> {
       let txstore = self.hash_to_txstore.remove(hash)?;
       for input in &txstore.signed_tx.tx.tx_input {
        self.input_to_hash.remove(&input);
       }
       self.index_to_hash.remove(&txstore.index);
       Some(txstore)
    }
    
    // when a new propser block is found, txs(tx1) in it must be deleted from mempool
    //other txs(tx2) that use the same inputs as tx1 must be deleted
    // this func is used to remove other txs that use "to be deleted tx's output(tx2)" as input
    pub fn delete_dependent_txs(&mut self, txoutput: &UtxoInput){

    //makes recursively calls,should i use aa queue to simulate recursion instead?
            if let Some(txstore_hash) = self.input_to_hash.get(&txoutput) {
                let txstore_hash = *txstore_hash;
                let txstore = self.delete_and_get(&txstore_hash).unwrap();
                for (index, output) in txstore.signed_tx.tx.tx_output.iter().enumerate() {
                    let input = UtxoInput {
                        
                            tx_hash: txstore_hash,
                            idx: index as u8,
                        
                    };
                   self. delete_dependent_txs(&input);
                }
            }
    }

    
    /// get n transaction in fifo order
    pub fn get_transactions(&self, n: u32) -> Vec<SignedTransaction> {
        let result:Vec<SignedTransaction> = vec![];
        if n > self.mempool_len().try_into().unwrap(){
            return vec![];
        }
        for i  in 0..n {
            let hash = self.index_to_hash.get(&i).unwrap();
            let txstore = self.hash_to_txstore.get(hash).unwrap();
            result.push(txstore.signed_tx);
        }
        result
    }
    

    pub fn mempool_len(&self) -> usize {
        self.hash_to_txstore.len()
    }

}
  
