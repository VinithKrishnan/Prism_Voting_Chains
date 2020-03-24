use crate::transaction::{self, SignedTransaction};

pub fn is_tx_valid(signed_tx: &SignedTransaction) -> bool {
   //verify whether the tx is signed properly
   return transaction::verify(&signed_tx.tx, &signed_tx.signature, &signed_tx.public_key);
}
