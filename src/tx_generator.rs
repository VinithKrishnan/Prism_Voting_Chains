use crate::network::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::ledger_state::{BlockState,State};
use crate::block::*;
use crate::transaction::{self,*};
use crate::crypto::hash::{H256, Hashable};
use crate::crypto::address::{*};
use crate::network::message::Message;
use log::{debug,info};
use rand::Rng;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use ring::signature::{self,Ed25519KeyPair, Signature, KeyPair};
use crate::mempool::TransactionMempool;
use crate::crypto::key_pair;
use crate::crypto::address::{self,*};
use std::borrow::Borrow;
use std::collections::HashMap;


extern crate chrono;
use chrono::prelude::*;

use std::time;
use std::thread;
use std::sync::{Arc, Mutex};

enum ControlSignal {
    Start(u64,u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64,u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    mempool: Arc<Mutex<TransactionMempool>>,
    ledger_state: Arc<Mutex<BlockState>>,
    blockchain: Arc<Mutex<Blockchain>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,

}

pub fn new(
    server: &ServerHandle,
    mempool: &Arc<Mutex<TransactionMempool>>,
    ledger_state: &Arc<Mutex<BlockState>>,
    blockchain: &Arc<Mutex<Blockchain>>
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        mempool: Arc::clone(mempool),
        ledger_state: Arc::clone(ledger_state),
        blockchain: Arc::clone(blockchain),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64,index: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda,index))
            .unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("tx_generator".to_string())
            .spawn(move || {
                self.gen_loop();
            })
            .unwrap();
        info!("Generator initialized into paused mode");
    }

    fn handle_control_signal(&mut self, signal: ControlSignal) {
        match signal {
            ControlSignal::Exit => {
                info!("Generator shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i,j) => {
                info!("Generator starting in continuous mode with lambda {} and index {}", i,j);
                self.operating_state = OperatingState::Run(i,j);
            }
        }
    }

    fn gen_loop(&mut self) {
        // main mining loop
        // public_key:key.public_key().as_ref().to_vec()

        /*let public_key1: Vec<u8> = b"AAAAC3NzaC1lZDI1NTE5AAAAICYqyx/qrxvVPB2lPvV3ZmTH+uYwB6wL1hkBlGaYPmGu".to_vec();
        let public_key2: Vec<u8> = b"AAAAC3NzaC1lZDI1NTE5AAAAIDfqgH+ezyswXrz2YNDkkYXCTCTMi+Ms6GWW5NQXNUc4".to_vec();
        let public_key3: Vec<u8> = b"AAAAC3NzaC1lZDI1NTE5AAAAIMborH2X51+g+ziV0LmZY8p90+eEP/9jPAOUauBPorL/".to_vec();
  
  
        let address1 = address::address_from_public_key_vec_ref(&public_key1);
        let address2 = address::address_from_public_key_vec_ref(&public_key2);
        let address3 = address::address_from_public_key_vec_ref(&public_key3);*/

        let key_pair1 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 187, 131, 74, 161, 134, 11, 240, 6, 188, 109, 18, 108, 124, 219, 167, 164, 215, 125, 168, 79, 204, 194, 232, 91, 58, 186, 181, 230, 212, 78, 163, 28, 161, 35, 3, 33, 0, 233, 72, 146, 218, 220, 235, 17, 123, 202, 112, 119, 63, 134, 105, 134, 71, 34, 185, 71, 193, 59, 66, 43, 137, 50, 194, 120, 234, 97, 132, 235, 159].as_ref().into()).unwrap();
        let key_pair2 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 154, 186, 73, 239, 105, 129, 142, 211, 156, 79, 213, 209, 229, 87, 22, 92, 113, 203, 244, 222, 244, 33, 199, 254, 130, 102, 178, 65, 198, 67, 20, 132, 161, 35, 3, 33, 0, 161, 153, 171, 27, 96, 146, 25, 237, 5, 189, 186, 116, 0, 24, 2, 8, 28, 143, 5, 119, 20, 47, 142, 186, 55, 234, 189, 167, 154, 15, 210, 97].as_ref().into()).unwrap();
        let key_pair3 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 147, 195, 231, 118, 135, 29, 32, 40, 23, 117, 107, 218, 6, 220, 198, 50, 81, 113, 167, 122, 175, 161, 118, 93, 191, 137, 50, 125, 203, 69, 70, 42, 161, 35, 3, 33, 0, 125, 80, 160, 138, 247, 46, 227, 162, 118, 51, 64, 42, 174, 60, 87, 134, 77, 60, 225, 11, 189, 222, 22, 185, 65, 10, 67, 78, 250, 41, 188, 60].as_ref().into()).unwrap();
        
        let vector1 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 187, 131, 74, 161, 134, 11, 240, 6, 188, 109, 18, 108, 124, 219, 167, 164, 215, 125, 168, 79, 204, 194, 232, 91, 58, 186, 181, 230, 212, 78, 163, 28, 161, 35, 3, 33, 0, 233, 72, 146, 218, 220, 235, 17, 123, 202, 112, 119, 63, 134, 105, 134, 71, 34, 185, 71, 193, 59, 66, 43, 137, 50, 194, 120, 234, 97, 132, 235, 159];
        let vector2 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 154, 186, 73, 239, 105, 129, 142, 211, 156, 79, 213, 209, 229, 87, 22, 92, 113, 203, 244, 222, 244, 33, 199, 254, 130, 102, 178, 65, 198, 67, 20, 132, 161, 35, 3, 33, 0, 161, 153, 171, 27, 96, 146, 25, 237, 5, 189, 186, 116, 0, 24, 2, 8, 28, 143, 5, 119, 20, 47, 142, 186, 55, 234, 189, 167, 154, 15, 210, 97];
        let vector3 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 147, 195, 231, 118, 135, 29, 32, 40, 23, 117, 107, 218, 6, 220, 198, 50, 81, 113, 167, 122, 175, 161, 118, 93, 191, 137, 50, 125, 203, 69, 70, 42, 161, 35, 3, 33, 0, 125, 80, 160, 138, 247, 46, 227, 162, 118, 51, 64, 42, 174, 60, 87, 134, 77, 60, 225, 11, 189, 222, 22, 185, 65, 10, 67, 78, 250, 41, 188, 60];
        
        

        let address1 = address::address_from_public_key_vec_ref(&key_pair1.public_key().as_ref().to_vec());
        let address2 = address::address_from_public_key_vec_ref(&key_pair2.public_key().as_ref().to_vec());
        let address3 = address::address_from_public_key_vec_ref(&key_pair3.public_key().as_ref().to_vec());
  
        
        let mut index:u64 = 0;
        let mut time_i:u64 = 0;
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    self.handle_control_signal(signal);
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        self.handle_control_signal(signal);
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Generator control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            if let OperatingState::Run(i,j) = self.operating_state {
                index = j;
                time_i =i;  
            }


        // actual transaction generation

        //getting tip state
        let locked_state = self.ledger_state.lock().unwrap();
        let locked_blockchain = self.blockchain.lock().unwrap();
        let mut locked_mempool = self.mempool.lock().unwrap();
        let tiphash = locked_blockchain.tiphash;
        let mut map_state:HashMap<UtxoInput, UtxoOutput> = HashMap::new();
        let mut tip_state:State = State{state_map:map_state};
        if locked_state.block_state_map.contains_key(&tiphash){
        tip_state = locked_state.block_state_map.get(&tiphash).unwrap().clone();
        }
        else{
            info!("Tiphash not present in state_map");
        }

        let mut ref_addr:H160=generate_random_address() ;
        let mut send_addr:H160=generate_random_address() ;
        let mut key:signature::Ed25519KeyPair=key_pair::random();
        match index {
            0 => {ref_addr = address1;send_addr=address3;key = signature::Ed25519KeyPair::from_pkcs8(vector1.as_ref().into()).unwrap(); },
            1 => {ref_addr = address2;send_addr=address3;key = signature::Ed25519KeyPair::from_pkcs8(vector2.as_ref().into()).unwrap(); },
            2 => {ref_addr = address3;send_addr=address3;key = signature::Ed25519KeyPair::from_pkcs8(vector3.as_ref().into()).unwrap(); },
            _ => println!("Invalid index"),
        }
        
        
        let mut tx_buffer : Vec<H256> = vec![];
        info!("About to generate tx");
        let mut balance:u32 = 0;
        for (input,output) in tip_state.state_map.iter() {
            if output.receipient_addr == ref_addr {
                let mut vec_input:Vec<UtxoInput> = vec![]; 
                let mut vec_output:Vec<UtxoOutput> = vec![];
                vec_input.push(input.clone());
                let mut new_output = output.clone();
                new_output.receipient_addr = send_addr;
                balance += new_output.value;
                vec_output.push(new_output);
                let t = Transaction{tx_input:vec_input,tx_output:vec_output};
                let sig = sign(&t, &key);
                let signed_tx = SignedTransaction{tx:t,signature:sig.as_ref().to_vec(),public_key:key.public_key().as_ref().to_vec()};
            
              //  if locked_mempool.tx_to_process.contains_key(&signed_tx.hash()){
               // continue;
              //  } else {
                tx_buffer.push(signed_tx.hash());
                println!("Adding transaction with hash {} to mempool in tx_generator",signed_tx.hash());
                locked_mempool.tx_to_process.insert(signed_tx.hash(),true);
                locked_mempool.tx_map.insert(signed_tx.hash(),signed_tx.clone());
                locked_mempool.tx_hash_queue.push_back(signed_tx.hash());
           //    }
                }
        }
        info!("Balance of this node is: {}",balance);
        /*for x in 0..10 {
            let t = transaction::generate_random_transaction();
            let key = key_pair::random();
            let sig = sign(&t, &key);
            let signed_tx = SignedTransaction{tx:t,signature:sig.as_ref().to_vec(),public_key:key.public_key().as_ref().to_vec()};
            //println!("generated signed transaction with hash {} in tx_generator",signed_tx.hash());
            //locked_mempool.insert(t.hash(),t);
            if locked_mempool.tx_to_process.contains_key(&signed_tx.hash()){
                continue;
            } else {
                tx_buffer.push(signed_tx.hash());
                println!("Adding transaction with hash {} to mempool in tx_generator",signed_tx.hash());
                locked_mempool.tx_to_process.insert(signed_tx.hash(),true);
                locked_mempool.tx_map.insert(signed_tx.hash(),signed_tx.clone());
                locked_mempool.tx_hash_queue.push_back(signed_tx.hash());
            }
            
        }*/
        if tx_buffer.len()>0 {
        self.server.broadcast(Message::NewTransactionHashes(tx_buffer));
        }
        std::mem::drop(locked_mempool);
        std::mem::drop(locked_blockchain);
        std::mem::drop(locked_state);
        





        /*if let OperatingState::Run(i,j) = self.operating_state {
            if i != 0 {
                let interval = time::Duration::from_micros(i as u64);
                thread::sleep(interval);
            }
        }*/

        if time_i != 0 {
            let interval = time::Duration::from_micros(time_i);
            thread::sleep(interval);
        }
        

        }
    }
}