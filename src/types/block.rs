use blake2::digest::FixedOutput;
use blake2::{Blake2s, Digest};

use crate::traits::{Hashable, Verifiable};
use crate::types::{Hash, Transaction};

#[derive(Debug, Default, Clone)]
pub struct Block {
    nonce: u128,
    pub(crate) hash: Option<Hash>,
    pub(crate) prev_hash: Option<Hash>,
    pub(crate) transactions: Vec<Transaction>,
}

impl Hashable for Block {
    fn hash(&self) -> Hash {
        let mut hasher = Blake2s::new();

        hasher.update(format!("{:?}", (self.prev_hash.clone(), self.nonce)).as_bytes());
        for tx in self.transactions.iter() {
            hasher.update(tx.hash())
        }

        hex::encode(hasher.finalize_fixed())
    }
}

impl Verifiable for Block {
    fn verify(&self) -> bool {
        matches!(&self.hash, Some(hash) if hash == &self.hash())
    }
}

impl Block {
    pub fn new(prev_hash: Option<Hash>) -> Self {
        Block {
            prev_hash,
            ..Default::default()
        }
    }

    pub fn set_nonce(&mut self, nonce: u128) {
        self.nonce = nonce;
        self.update_hash();
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        self.transactions.push(tx);
        self.update_hash();
    }

    pub fn transactions_len(&self) -> usize {
        self.transactions.len()
    }

    fn update_hash(&mut self) {
        self.hash = Some(self.hash())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::create_account_tx;

    #[test]
    fn test_couple_blocks() {
        let mut block1 = Block::new(None);
        block1.set_nonce(1);
        let block2 = Block::new(Some(block1.hash()));

        assert_eq!(block2.prev_hash, block1.hash);
    }

    #[test]
    fn test_empty_txs() {
        let block = Block::new(None);

        assert!(block.hash.is_none());
        assert_eq!(block.transactions_len(), 0);
    }

    #[test]
    fn test_add_tx() {
        let mut block = Block::new(None);
        let tx = create_account_tx("alice".to_string());
        block.add_transaction(tx.clone());
        block.add_transaction(tx);

        assert!(block.hash.is_some());
        assert_eq!(block.transactions_len(), 2);
    }

    #[test]
    fn test_hash() {
        let mut block = Block::new(None);
        block.set_nonce(1);

        assert_eq!(
            block.hash(),
            "498e136dc59a854b899c330839ca431dd737016530957341966e043162bc8af7"
        );
        assert_eq!(block.hash(), block.hash.unwrap());
    }

    #[test]
    fn test_block_verify_passed() {
        let mut block = Block::new(None);
        block.set_nonce(1);

        assert!(block.verify());
    }

    #[test]
    fn test_block_verify_failed() {
        let mut block = Block::new(None);
        block.set_nonce(1);
        block.nonce = 2;

        assert!(!block.verify());
    }
}
