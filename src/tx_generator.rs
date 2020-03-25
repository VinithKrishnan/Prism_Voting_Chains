use crate::network::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::block::*;
use crate::transaction::{self,*};
use crate::crypto::hash::{H256, Hashable};
use crate::network::message::Message;
use log::{debug,info};
use rand::Rng;
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use ring::signature::{self,Ed25519KeyPair, Signature, KeyPair};
use crate::mempool::TransactionMempool;
use crate::crypto::key_pair;

extern crate chrono;
use chrono::prelude::*;

use std::time;
use std::thread;
use std::sync::{Arc, Mutex};

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    mempool: Arc<Mutex<TransactionMempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,

}

pub fn new(
    server: &ServerHandle,
    mempool: &Arc<Mutex<TransactionMempool>>
) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        mempool: Arc::clone(mempool),
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

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
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
            ControlSignal::Start(i) => {
                info!("Generator starting in continuous mode with lambda {}", i);
                self.operating_state = OperatingState::Run(i);
            }
        }
    }

    fn gen_loop(&mut self) {
        // main mining loop
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
        // actual transaction generation
        let mut locked_mempool = self.mempool.lock().unwrap();
        let mut tx_buffer : Vec<H256> = vec![];
        for x in 0..10 {
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
            
        }
        self.server.broadcast(Message::NewTransactionHashes(tx_buffer));
        std::mem::drop(locked_mempool);
        





        if let OperatingState::Run(i) = self.operating_state {
            if i != 0 {
                let interval = time::Duration::from_micros(i as u64);
                thread::sleep(interval);
            }
        }
        

        }
    }
}