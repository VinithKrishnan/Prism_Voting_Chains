extern crate bincode;
extern crate serde;

use serde::{Serialize,Deserialize};
use ring::signature::{self,Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use rand::Rng;
use crate::crypto::hash::{H256, Hashable};

#[derive(Serialize, Deserialize, Debug, Default,Clone)]
pub struct Transaction {
    input : Vec<u8>,
    output : Vec<u8>,
}

//pub fn pr(){
//    println!("hello");
//}

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
    /*Default::default();*/
    let mut rng = rand::thread_rng();
    let mut random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let input_bytes = random_bytes;
    random_bytes = (0..32).map(|_| rng.gen()).collect();
    let output_bytes = random_bytes;
    Transaction{input : input_bytes,output : output_bytes}
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
