#[cfg(test)]
#[macro_use]

use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::transaction::Transaction;
use crate::transaction::tests::generate_random_transaction;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Header {
    pub parenthash: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkle_root:H256,
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Content {
    pub data:Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Block {
    pub header:Header,
    pub content:Content,
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        let encodedhead: Vec<u8> = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &encodedhead[..]).into()
    }
}


#[cfg(any(test, test_utilities))]
pub mod test {
    use super::*;
    use crate::crypto::hash::H256;

    pub fn generate_random_block(parent: &H256) -> Block {
        let mut rng = rand::thread_rng();
        let r1:u32 = rng.gen();
        let r2:u128 = rng.gen();
        let mut buffer: [u8; 32] = [0; 32];
        let b:H256 = buffer.into();
        let h:Header = Header{parenthash:*parent,nonce:r1,difficulty:b,timestamp:r2,merkle_root:b};
        let t = generate_random_transaction();
        //transaction::pr();
        let mut vect:Vec<Transaction> = vec![];
        vect.push(t);
        let c:Content = Content{data:vect};
        let b:Block = Block{header:h,content:c};
        b
    }
}
