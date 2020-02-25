use serde::{Serialize, Deserialize};
use crate::crypto::hash::{H256, Hashable};
use crate::block::{self, *};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    NewBlockHashes(Vec<H256>),
    GetBlocks(Vec<H256>),
    Blocks(Vec<Block>),
}
