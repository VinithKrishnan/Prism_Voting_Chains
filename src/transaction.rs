extern crate bincode;
extern crate serde;

use serde::{Serialize,Deserialize};
use ring::signature::{self,Ed25519KeyPair, Signature, KeyPair};

use crate::crypto::hash::{self, H256, Hashable};
use crate::crypto::address::{self, H160};

#[derive(Serialize, Deserialize, Debug, Default,Clone)]
pub struct UtxoInput{
  pub tx_hash: H256,
  pub idx: u8,  
}

#[derive(Serialize, Deserialize, Debug, Default,Clone)]
pub struct UtxoOutput{
  pub receipient_addr: H160,
  pub value: u32,
}

#[derive(Serialize, Deserialize, Debug, Default,Clone)]
pub struct Transaction {
  pub tx_input: Vec<UtxoInput>,
  pub tx_output: Vec<UtxoOutput>,
}

#[derive(Serialize, Deserialize, Debug, Default,Clone)]
pub struct SignedTransaction {
  pub tx: Transaction,
  pub signature: Vec<u8>, 
  pub public_key: Vec<u8>,
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let encodedtrans: Vec<u8> = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &encodedtrans[..]).into()
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    //let mut mess = [&t.input[..],&t.output[..]].concat();
    let encoded: Vec<u8> = bincode::serialize(&t).unwrap();
    //let merged : Vec<_> =mess.iter().flat_map(|s.as_mut()| s.iter()).collect();
    let sig = key.sign(&encoded[..]);
    sig
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &<Ed25519KeyPair as KeyPair>::PublicKey, signature: &Signature) -> bool {
    //let mut mess = [&t.input[..],&t.output[..]].concat();
    let encoded: Vec<u8> = bincode::serialize(&t).unwrap();
    let public_key_bytes = public_key.as_ref();
    let peer_public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key_bytes);
    peer_public_key.verify(&encoded[..],signature.as_ref()).is_ok()
}


pub fn generate_random_transaction() -> Transaction {
    let input = vec![UtxoInput{tx_hash: hash::generate_random_hash(), idx: 0}];
    let output = vec![UtxoOutput{receipient_addr: address::generate_random_address(), value: 0}];
    
    Transaction{tx_input: input, tx_output: output}
}

pub fn generate_genesis_transaction() -> Transaction {
    let input = vec![UtxoInput{tx_hash: H256::from([0;32]), idx: 0}];
    let output = vec![UtxoOutput{receipient_addr: H160::from([0;20]), value: 0}];
    
    Transaction{tx_input: input, tx_output: output}
}


#[cfg(any(test, test_utilities))]
pub mod tests {
    use super::*;
    use crate::crypto::key_pair;

/*
    pub fn generate_random_transaction() -> Transaction {
        /*Default::default();*/
        let mut rng = rand::thread_rng();
        let mut random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let input_bytes = random_bytes;
        random_bytes = (0..32).map(|_| rng.gen()).collect();
        let output_bytes = random_bytes;
        Transaction{input : input_bytes,output : output_bytes}
    }*/


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, &(key.public_key()), &signature));
    }
}
