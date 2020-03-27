use crate::transaction::{UtxoInput, UtxoOutput};
use crate::crypto::hash::H256;
use crate::block::Block;
use crate::crypto::hash::Hashable;
use crate::crypto::address::{self, H160};
use ring::signature::{self,Ed25519KeyPair, Signature, KeyPair};

use std::collections::HashMap;
use log::debug;
use log::info;

#[derive(Debug, Default, Clone)]
pub struct State{
    pub state_map: HashMap<UtxoInput, UtxoOutput>,  
}

pub struct BlockState{
    pub block_state_map: HashMap<H256, State>,
}

impl BlockState {
    pub fn new(genhash:&H256)-> Self{
      let gen_state = ico();
      let mut hmap:HashMap<H256, State>= HashMap::new();
      hmap.insert(*genhash,gen_state.clone());
      let bs_map:BlockState = BlockState{block_state_map:hmap};
      info!("ICO has been added to the state of genesis block.");
      bs_map
    }
}

//TODO: Can change the signature of the function if passing some mutex version
//of block_state
pub fn update_block_state(block: &Block, block_state: &mut BlockState) {
  let parent_state = &block_state.block_state_map[&block.header.parenthash];
  let mut cur_block_state = parent_state.clone();
  
  //println!{"parent_state {:?}", parent_state}

  for signed_tx in &block.content.data {
      for tx_input in &signed_tx.tx.tx_input {
          cur_block_state.state_map.remove(tx_input);
      }
      for (i, tx_output) in (&signed_tx.tx.tx_output).iter().enumerate() {
          let tx_input = UtxoInput{tx_hash: signed_tx.tx.hash(), idx: i as u8};
          cur_block_state.state_map.insert(tx_input, tx_output.clone());
      }
  }
  println!("UTXO state of current block:");
  let addr_val_map:HashMap<H160,u32> = HashMap::new();
  for (key,value) in cur_block_state.state_map.iter() {
    println!("{:?}{:?}", key,value);
}
  
  //println!{"cur_block_state {:?}", cur_block_state}
  block_state.block_state_map.insert(block.hash(), cur_block_state);
  
}

pub fn ico() -> State {
  /*let public_key1: Vec<u8> = b"AAAAC3NzaC1lZDI1NTE5AAAAICYqyx/qrxvVPB2lPvV3ZmTH+uYwB6wL1hkBlGaYPmGu".to_vec();
  let public_key2: Vec<u8> = b"AAAAC3NzaC1lZDI1NTE5AAAAIDfqgH+ezyswXrz2YNDkkYXCTCTMi+Ms6GWW5NQXNUc4".to_vec();
  let public_key3: Vec<u8> = b"AAAAC3NzaC1lZDI1NTE5AAAAIMborH2X51+g+ziV0LmZY8p90+eEP/9jPAOUauBPorL/".to_vec();
  
  let mut address_vec: Vec<H160> = vec![];
  let address1 = address::address_from_public_key_vec_ref(&public_key1);
  let address2 = address::address_from_public_key_vec_ref(&public_key2);
  let address3 = address::address_from_public_key_vec_ref(&public_key3);
  
  address_vec.push(address1);
  address_vec.push(address2);
  address_vec.push(address3);*/

  let key_pair1 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 187, 131, 74, 161, 134, 11, 240, 6, 188, 109, 18, 108, 124, 219, 167, 164, 215, 125, 168, 79, 204, 194, 232, 91, 58, 186, 181, 230, 212, 78, 163, 28, 161, 35, 3, 33, 0, 233, 72, 146, 218, 220, 235, 17, 123, 202, 112, 119, 63, 134, 105, 134, 71, 34, 185, 71, 193, 59, 66, 43, 137, 50, 194, 120, 234, 97, 132, 235, 159].as_ref().into()).unwrap();
  let key_pair2 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 154, 186, 73, 239, 105, 129, 142, 211, 156, 79, 213, 209, 229, 87, 22, 92, 113, 203, 244, 222, 244, 33, 199, 254, 130, 102, 178, 65, 198, 67, 20, 132, 161, 35, 3, 33, 0, 161, 153, 171, 27, 96, 146, 25, 237, 5, 189, 186, 116, 0, 24, 2, 8, 28, 143, 5, 119, 20, 47, 142, 186, 55, 234, 189, 167, 154, 15, 210, 97].as_ref().into()).unwrap();
  let key_pair3 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 147, 195, 231, 118, 135, 29, 32, 40, 23, 117, 107, 218, 6, 220, 198, 50, 81, 113, 167, 122, 175, 161, 118, 93, 191, 137, 50, 125, 203, 69, 70, 42, 161, 35, 3, 33, 0, 125, 80, 160, 138, 247, 46, 227, 162, 118, 51, 64, 42, 174, 60, 87, 134, 77, 60, 225, 11, 189, 222, 22, 185, 65, 10, 67, 78, 250, 41, 188, 60].as_ref().into()).unwrap();
  let key_pair4 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 11, 212, 170, 1, 126, 8, 32, 58, 40, 116, 165, 98, 48, 127, 67, 109, 86, 251, 249, 203, 244, 203, 1, 223, 248, 164, 176, 195, 23, 17, 146, 8, 161, 35, 3, 33, 0, 206, 15, 234, 106, 58, 45, 177, 81, 0, 193, 13, 113, 249, 55, 152, 151, 227, 224, 35, 185, 148, 49, 186, 234, 17, 106, 132, 216, 83, 196, 127, 99].as_ref().into()).unwrap();  
  let key_pair5 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 40, 29, 27, 179, 25, 183, 68, 113, 252, 19, 20, 114, 160, 221, 228, 195, 253, 87, 245, 176, 226, 99, 249, 28, 87, 61, 101, 129, 207, 87, 90, 195, 161, 35, 3, 33, 0, 254, 57, 159, 24, 159, 141, 184, 159, 58, 86, 112, 217, 153, 215, 65, 7, 88, 14, 57, 80, 42, 33, 151, 211, 208, 52, 42, 208, 111, 174, 223, 27].as_ref().into()).unwrap();
  let key_pair6 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 224, 231, 169, 219, 160, 221, 218, 51, 189, 197, 202, 218, 24, 20, 166, 105, 31, 55, 241, 231, 5, 165, 51, 106, 174, 11, 110, 84, 17, 115, 230, 56, 161, 35, 3, 33, 0, 127, 130, 60, 237, 224, 179, 64, 241, 25, 174, 45, 64, 52, 179, 70, 249, 26, 49, 128, 103, 188, 201, 48, 55, 221, 154, 12, 83, 40, 123, 3, 157].as_ref().into()).unwrap();

  let mut address_vec: Vec<H160> = vec![];
  let address1 = address::address_from_public_key_vec_ref(&key_pair1.public_key().as_ref().to_vec());
  let address2 = address::address_from_public_key_vec_ref(&key_pair2.public_key().as_ref().to_vec());
  let address3 = address::address_from_public_key_vec_ref(&key_pair3.public_key().as_ref().to_vec());
  let address4 = address::address_from_public_key_vec_ref(&key_pair4.public_key().as_ref().to_vec());
  let address5 = address::address_from_public_key_vec_ref(&key_pair5.public_key().as_ref().to_vec());
  let address6 = address::address_from_public_key_vec_ref(&key_pair6.public_key().as_ref().to_vec());
  
  address_vec.push(address1);
  address_vec.push(address2);
  address_vec.push(address3);
  address_vec.push(address4);
  address_vec.push(address5);
  address_vec.push(address6);

  let mut sam = hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920");
  
  //let mut initial_tx_hash: H256 = sam.into() ;
  let val: u32 = 1000;
  
  let mut initial_state: State = State{state_map: HashMap::new()};
  
  //sam[j/10]=(j%10) as u8;
  //let mut initial_tx_hash: H256 = sam.into() ;
  for (i,address) in  address_vec.iter().enumerate() {
    for j in 1..3{
    sam[j/10]=i as u8 + 10*j as u8;
    let mut initial_tx_hash: H256 = sam.into() ;
    let input = UtxoInput{tx_hash: initial_tx_hash, idx: 0};
    let output = UtxoOutput{receipient_addr: *address, value: val};
    initial_state.state_map.insert(input, output);
    }
  }
  
  
  for (key,value) in initial_state.state_map.iter() {
    println!("{:?}{:?}", key,value);
  }

  let mut utxo_hmap:HashMap<H160,u32> = HashMap::new();

  for (utxo_input,utxo_output) in initial_state.state_map.iter() {
      if !utxo_hmap.contains_key(&utxo_output.receipient_addr){
          utxo_hmap.insert(utxo_output.receipient_addr,utxo_output.value);
      }else{
          *utxo_hmap.get_mut(&utxo_output.receipient_addr).unwrap() = *utxo_hmap.get_mut(&utxo_output.receipient_addr).unwrap()+utxo_output.value;
      }
  }
  println!("Balance in accounts after ico:");
  for (key,value) in utxo_hmap.iter() {
      println!("balance in addr {:?} is {:?}",key,value);
  }
   
  initial_state
}
