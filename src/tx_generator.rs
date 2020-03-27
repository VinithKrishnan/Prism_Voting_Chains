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
                println!("Generator shutting down");
                self.operating_state = OperatingState::ShutDown;
            }
            ControlSignal::Start(i,j) => {
                println!("Generator starting in continuous mode with lambda {} and index {}", i,j);
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
        let key_pair4 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 11, 212, 170, 1, 126, 8, 32, 58, 40, 116, 165, 98, 48, 127, 67, 109, 86, 251, 249, 203, 244, 203, 1, 223, 248, 164, 176, 195, 23, 17, 146, 8, 161, 35, 3, 33, 0, 206, 15, 234, 106, 58, 45, 177, 81, 0, 193, 13, 113, 249, 55, 152, 151, 227, 224, 35, 185, 148, 49, 186, 234, 17, 106, 132, 216, 83, 196, 127, 99].as_ref().into()).unwrap();
        let key_pair5 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 40, 29, 27, 179, 25, 183, 68, 113, 252, 19, 20, 114, 160, 221, 228, 195, 253, 87, 245, 176, 226, 99, 249, 28, 87, 61, 101, 129, 207, 87, 90, 195, 161, 35, 3, 33, 0, 254, 57, 159, 24, 159, 141, 184, 159, 58, 86, 112, 217, 153, 215, 65, 7, 88, 14, 57, 80, 42, 33, 151, 211, 208, 52, 42, 208, 111, 174, 223, 27].as_ref().into()).unwrap();
        let key_pair6 = signature::Ed25519KeyPair::from_pkcs8([48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 224, 231, 169, 219, 160, 221, 218, 51, 189, 197, 202, 218, 24, 20, 166, 105, 31, 55, 241, 231, 5, 165, 51, 106, 174, 11, 110, 84, 17, 115, 230, 56, 161, 35, 3, 33, 0, 127, 130, 60, 237, 224, 179, 64, 241, 25, 174, 45, 64, 52, 179, 70, 249, 26, 49, 128, 103, 188, 201, 48, 55, 221, 154, 12, 83, 40, 123, 3, 157].as_ref().into()).unwrap();

        let vector1 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 187, 131, 74, 161, 134, 11, 240, 6, 188, 109, 18, 108, 124, 219, 167, 164, 215, 125, 168, 79, 204, 194, 232, 91, 58, 186, 181, 230, 212, 78, 163, 28, 161, 35, 3, 33, 0, 233, 72, 146, 218, 220, 235, 17, 123, 202, 112, 119, 63, 134, 105, 134, 71, 34, 185, 71, 193, 59, 66, 43, 137, 50, 194, 120, 234, 97, 132, 235, 159];
        let vector2 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 154, 186, 73, 239, 105, 129, 142, 211, 156, 79, 213, 209, 229, 87, 22, 92, 113, 203, 244, 222, 244, 33, 199, 254, 130, 102, 178, 65, 198, 67, 20, 132, 161, 35, 3, 33, 0, 161, 153, 171, 27, 96, 146, 25, 237, 5, 189, 186, 116, 0, 24, 2, 8, 28, 143, 5, 119, 20, 47, 142, 186, 55, 234, 189, 167, 154, 15, 210, 97];
        let vector3 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 147, 195, 231, 118, 135, 29, 32, 40, 23, 117, 107, 218, 6, 220, 198, 50, 81, 113, 167, 122, 175, 161, 118, 93, 191, 137, 50, 125, 203, 69, 70, 42, 161, 35, 3, 33, 0, 125, 80, 160, 138, 247, 46, 227, 162, 118, 51, 64, 42, 174, 60, 87, 134, 77, 60, 225, 11, 189, 222, 22, 185, 65, 10, 67, 78, 250, 41, 188, 60];
        let vector4 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 11, 212, 170, 1, 126, 8, 32, 58, 40, 116, 165, 98, 48, 127, 67, 109, 86, 251, 249, 203, 244, 203, 1, 223, 248, 164, 176, 195, 23, 17, 146, 8, 161, 35, 3, 33, 0, 206, 15, 234, 106, 58, 45, 177, 81, 0, 193, 13, 113, 249, 55, 152, 151, 227, 224, 35, 185, 148, 49, 186, 234, 17, 106, 132, 216, 83, 196, 127, 99];
        let vector5 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 40, 29, 27, 179, 25, 183, 68, 113, 252, 19, 20, 114, 160, 221, 228, 195, 253, 87, 245, 176, 226, 99, 249, 28, 87, 61, 101, 129, 207, 87, 90, 195, 161, 35, 3, 33, 0, 254, 57, 159, 24, 159, 141, 184, 159, 58, 86, 112, 217, 153, 215, 65, 7, 88, 14, 57, 80, 42, 33, 151, 211, 208, 52, 42, 208, 111, 174, 223, 27];
        let vector6 = [48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 224, 231, 169, 219, 160, 221, 218, 51, 189, 197, 202, 218, 24, 20, 166, 105, 31, 55, 241, 231, 5, 165, 51, 106, 174, 11, 110, 84, 17, 115, 230, 56, 161, 35, 3, 33, 0, 127, 130, 60, 237, 224, 179, 64, 241, 25, 174, 45, 64, 52, 179, 70, 249, 26, 49, 128, 103, 188, 201, 48, 55, 221, 154, 12, 83, 40, 123, 3, 157];

        let address1 = address::address_from_public_key_vec_ref(&key_pair1.public_key().as_ref().to_vec());
        let address2 = address::address_from_public_key_vec_ref(&key_pair2.public_key().as_ref().to_vec());
        let address3 = address::address_from_public_key_vec_ref(&key_pair3.public_key().as_ref().to_vec());
        let address4 = address::address_from_public_key_vec_ref(&key_pair4.public_key().as_ref().to_vec());
        let address5 = address::address_from_public_key_vec_ref(&key_pair5.public_key().as_ref().to_vec());
        let address6 = address::address_from_public_key_vec_ref(&key_pair6.public_key().as_ref().to_vec());
        
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
        let locked_blockchain = self.blockchain.lock().unwrap();
        let mut locked_mempool = self.mempool.lock().unwrap();
        let locked_state = self.ledger_state.lock().unwrap();
       
        
        let tiphash = locked_blockchain.tiphash;
        let mut map_state:HashMap<UtxoInput, UtxoOutput> = HashMap::new();
        let mut tip_state:State = State{state_map:map_state};
        if locked_state.block_state_map.contains_key(&tiphash){
        tip_state = locked_state.block_state_map.get(&tiphash).unwrap().clone();
        }
        else{
            println!("Tiphash not present in state_map");
            continue;
        }
        for (key,value) in tip_state.state_map.iter() {
            println!("{:?},{:?}", key,value);
        }


        let mut ref_addr1:H160=generate_random_address() ;
        let mut ref_addr2:H160=generate_random_address() ;
        let mut send_addr:H160=generate_random_address() ;
        let mut key1:signature::Ed25519KeyPair=key_pair::random();
        let mut key2:signature::Ed25519KeyPair=key_pair::random();
        match index {
            0 => {ref_addr1 = address1;send_addr=address2;ref_addr2 = address2;key1 = signature::Ed25519KeyPair::from_pkcs8(vector1.as_ref().into()).unwrap();key2 = signature::Ed25519KeyPair::from_pkcs8(vector2.as_ref().into()).unwrap(); },
            1 => {ref_addr1 = address3;send_addr=address4;ref_addr2 = address4;key1 = signature::Ed25519KeyPair::from_pkcs8(vector3.as_ref().into()).unwrap();key2 = signature::Ed25519KeyPair::from_pkcs8(vector4.as_ref().into()).unwrap(); },
            2 => {ref_addr1 = address5;send_addr=address6;ref_addr2 = address6;key1 = signature::Ed25519KeyPair::from_pkcs8(vector5.as_ref().into()).unwrap();key2 = signature::Ed25519KeyPair::from_pkcs8(vector6.as_ref().into()).unwrap(); },
           // 2 => {ref_addr = address3;send_addr=address3;key = signature::Ed25519KeyPair::from_pkcs8(vector3.as_ref().into()).unwrap(); },
            _ => println!("Invalid index"),
        }
        
        
        let mut tx_buffer : Vec<H256> = vec![];
        println!("About to generate tx");
        
        for (input,output) in tip_state.state_map.iter() {
            if output.receipient_addr == ref_addr1 || output.receipient_addr == ref_addr2  {
                let mut vec_input:Vec<UtxoInput> = vec![]; 
                let mut vec_output:Vec<UtxoOutput> = vec![];
                vec_input.push(input.clone());
                let mut new_output = output.clone();
                new_output.receipient_addr = send_addr;
               
                vec_output.push(new_output);
                let mut t = Transaction{tx_input:vec_input,tx_output:vec_output};
                let mut signed_tx = generate_random_signed_transaction();
                if output.receipient_addr == ref_addr1 {
                let sig = sign(&t, &key1);
                signed_tx = SignedTransaction{tx:t,signature:sig.as_ref().to_vec(),public_key:key1.public_key().as_ref().to_vec()};
                }else{
                let sig = sign(&t, &key2);
                signed_tx = SignedTransaction{tx:t,signature:sig.as_ref().to_vec(),public_key:key2.public_key().as_ref().to_vec()};
                }
            
               if locked_mempool.tx_to_process.contains_key(&signed_tx.hash()){
               continue;
               } else {
                tx_buffer.push(signed_tx.hash());
                println!("Adding transaction with recepient address {:?} to mempool in tx_generator",send_addr);
                locked_mempool.tx_to_process.insert(signed_tx.hash(),true);
                locked_mempool.tx_map.insert(signed_tx.hash(),signed_tx.clone());
                locked_mempool.tx_hash_queue.push_back(signed_tx.hash());
           //    }
                }
            }
        }
        //println!("Balance of account1 is {} and balance of account 2 is {}.",balance1,balance2);
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
        std::mem::drop(locked_state);
        std::mem::drop(locked_mempool);
        std::mem::drop(locked_blockchain);
        
        





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