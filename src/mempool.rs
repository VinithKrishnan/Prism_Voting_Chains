/*use crate::block::{self, *};
use crate::transaction::{self,*};
use crate::crypto::hash::{H256,Hashable};
use log::info;
use std::collections::HashMap;
use std::collections::VecDeque;

extern crate chrono;
use chrono::prelude::*;

pub struct Mempool {
    pub transq:VecDeque<H256>,
    pub bool_map:HashMap<H256,bool>,
    pub trans_map:HashMap<H256,SignedTransaction>,
}

impl Mempool{
    pub fn new() -> Self{
      Mempool{transq: VecDeque::new(), 
                         bool_map: HashMap::new(), 
                         trans_map: HashMap::new()}  
    }
  }
*/
use crate::crypto::hash::H256;
use crate::transaction::SignedTransaction;
  
use std::collections::VecDeque;
use std::collections::HashMap;
  
pub struct TransactionMempool{
    pub tx_hash_queue: VecDeque<H256>,
    pub tx_to_process: HashMap<H256, bool>,
    pub tx_map: HashMap<H256, SignedTransaction>,
}
  
impl TransactionMempool{
    pub fn new() -> Self{
      TransactionMempool{tx_hash_queue: VecDeque::new(), 
                         tx_to_process: HashMap::new(), 
                         tx_map: HashMap::new()}  
    }
}
  