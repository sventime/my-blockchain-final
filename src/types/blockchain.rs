use std::collections::hash_map::Entry;
use std::collections::HashMap;

use ed25519_dalek::PublicKey;

use crate::traits::{Hashable, Verifiable, WorldState};
use crate::types::account::Account;
use crate::types::chain::Chain;
use crate::types::{AccountId, AccountType, Block, Error, Hash, Transaction};

#[derive(Debug, Default, Clone)]
pub struct Blockchain {
    pub blocks: Chain<Block>,
    pub accounts: HashMap<AccountId, Account>,
    pub transactions_pool: Vec<Transaction>,
}

impl WorldState for Blockchain {
    fn get_account_ids(&self) -> Vec<AccountId> {
        self.accounts.keys().cloned().collect()
    }

    fn get_account_by_id(&self, id: &AccountId) -> Option<&Account> {
        self.accounts.get(id)
    }

    fn get_account_by_id_mut(&mut self, id: &AccountId) -> Option<&mut Account> {
        self.accounts.get_mut(id)
    }

    fn create_account(
        &mut self,
        account_id: AccountId,
        account_type: AccountType,
        public_key: PublicKey,
    ) -> Result<(), Error> {
        match self.accounts.entry(account_id.clone()) {
            Entry::Occupied(_) => Err(format!("AccountId already exist: {}", account_id)),
            Entry::Vacant(v) => {
                v.insert(Account::new(account_type, public_key));
                Ok(())
            }
        }
    }
}

impl Blockchain {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), Error> {
        if !block.verify() {
            return Err("Block has invalid hash".to_string());
        }

        let is_genesis = self.blocks.len() == 0;

        if !is_genesis && block.transactions.len() == 0 {
            return Err("Block has 0 transaction.".to_string());
        }

        let account_backup = self.accounts.clone();
        for transaction in block.transactions.clone() {
            let result = transaction.execute(self, is_genesis);
            if let Err(error) = result {
                self.accounts = account_backup;
                return Err(format!("Error during executing transactions: {}", error));
            }
        }

        self.blocks.append(block);
        Ok(())
    }

    pub fn get_last_block_hash(&self) -> Option<Hash> {
        self.blocks.head().map(|last_block| last_block.hash())
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut block_num = self.blocks.len();
        let mut prev_block_hash: Option<Hash> = None;

        for block in self.blocks.iter() {
            let is_genesis = block_num == 1;

            if !block.verify() {
                return Err(format!("Block {} has invalid hash", block_num));
            }

            if block.prev_hash.is_none() && !is_genesis {
                return Err(format!("Block {} doesn't have prev_hash", block_num));
            }

            if block.prev_hash.is_some() && is_genesis {
                return Err("Genesis block shouldn't have prev_hash".to_string());
            }

            if block_num != self.blocks.len() {
                if let Some(prev_block_hash) = &prev_block_hash {
                    if prev_block_hash != &block.hash.clone().unwrap() {
                        return Err(format!(
                            "Block {} prev_hash doesn't match Block {} hash",
                            block_num + 1,
                            block_num
                        ));
                    }
                }
            }

            prev_block_hash = block.prev_hash.clone();
            block_num -= 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::TransactionData;
    use crate::utils::{create_account_tx, generate_random_account};
    use ed25519_dalek::{Keypair, Signer};

    use super::*;

    fn append_block(bc: &mut Blockchain, nonce: u128) -> Block {
        let mut block = Block::new(bc.get_last_block_hash());
        block.set_nonce(nonce);
        block.add_transaction(create_account_tx(generate_random_account()));
        let block_clone = block.clone();
        assert!(bc.append_block(block).is_ok());
        block_clone
    }

    fn append_block_with_tx(
        bc: &mut Blockchain,
        nonce: u128,
        transactions: Vec<Transaction>,
    ) -> Result<Block, Error> {
        let mut block = Block::new(bc.get_last_block_hash());
        block.set_nonce(nonce);
        for transaction in transactions {
            block.add_transaction(transaction);
        }
        let block_clone = block.clone();
        bc.append_block(block)?;

        Ok(block_clone)
    }

    #[test]
    fn test_last_block_none() {
        assert_eq!(Blockchain::new().get_last_block_hash(), None);
    }

    #[test]
    fn test_last_block() {
        let mut bc = Blockchain::new();

        append_block(&mut bc, 1);
        append_block(&mut bc, 2);
        let last_block = append_block(&mut bc, 3);
        let last_block_hash = last_block.hash();

        assert_eq!(bc.len(), 3);
        assert_eq!(bc.get_last_block_hash(), Some(last_block_hash));
    }

    #[test]
    fn test_validate() {
        let bc = &mut Blockchain::new();

        append_block(bc, 1);
        append_block(bc, 2);
        append_block(bc, 3);
        bc.blocks.iter_mut().next().unwrap().transactions[0] =
            create_account_tx("malicios user".to_string());

        assert_eq!(bc.validate(), Err(String::from("Block 3 has invalid hash")));
    }

    #[test]
    fn test_validate_prev_hash() {
        let bc = &mut Blockchain::new();

        append_block(bc, 1);
        append_block(bc, 2);
        let block = bc.blocks.iter_mut().next().unwrap();
        block.prev_hash = Some("invalid_prev_hash".to_string());
        block.hash = Some(block.hash());

        assert_eq!(
            bc.validate(),
            Err(String::from("Block 2 prev_hash doesn't match Block 1 hash"))
        );
    }

    #[test]
    fn test_validate_prev_hash_none() {
        let bc = &mut Blockchain::new();

        append_block(bc, 1);
        append_block(bc, 2);
        let block = bc.blocks.iter_mut().next().unwrap();
        block.prev_hash = None;
        block.hash = Some(block.hash());

        assert_eq!(
            bc.validate(),
            Err(String::from("Block 2 doesn't have prev_hash"))
        );
    }

    #[test]
    fn test_append_without_tx() {
        let bc = &mut Blockchain::new();
        let mut block = Block::new(None);
        block.set_nonce(1);
        assert!(bc.append_block(block).is_ok());

        let mut block = Block::new(bc.get_last_block_hash());
        block.set_nonce(2);

        assert_eq!(
            bc.append_block(block),
            Err(String::from("Block has 0 transaction."))
        );
    }

    #[test]
    fn test_append_block_and_execute_tx() {
        let bc = &mut Blockchain::new();
        let mut block = Block::new(None);
        block.set_nonce(1);
        assert!(bc.append_block(block).is_ok());

        let tx = create_account_tx("alice".to_string());
        let mut block = Block::new(bc.get_last_block_hash());
        block.set_nonce(2);
        block.add_transaction(tx);
        assert!(bc.append_block(block).is_ok());

        let alice = bc.get_account_by_id(&"alice".to_string());
        assert!(alice.is_some());
        assert_eq!(alice.unwrap().account_type, AccountType::User);
    }

    #[test]
    fn test_rollback() {
        let bc = &mut Blockchain::new();
        let mut block = Block::new(None);
        block.set_nonce(1);
        assert!(bc.append_block(block).is_ok());

        let tx1 = create_account_tx("alice".to_string());
        let tx2 = create_account_tx("alice".to_string());
        let mut block = Block::new(bc.get_last_block_hash());
        block.set_nonce(2);
        block.add_transaction(tx1);
        block.add_transaction(tx2);
        assert_eq!(
            bc.append_block(block),
            Err("Error during executing transactions: AccountId already exist: alice".to_string())
        );

        let alice = bc.get_account_by_id(&"alice".to_string());
        assert!(alice.is_none());
    }

    #[test]
    fn test_initial_supply_fails() {
        let mut bc = Blockchain::new();

        let mut block = Block::new(None);
        block.set_nonce(1);
        block.add_transaction(Transaction::new(
            TransactionData::MintInitialSupply {
                to: "satoshi".to_string(),
                amount: 100_000_000,
            },
            None,
        ));
        assert_eq!(
            bc.append_block(block),
            Err("Error during executing transactions: Invalid account.".to_string())
        );
    }

    #[test]
    fn test_initial_supply_fails_if_not_genesis() {
        let bc = &mut Blockchain::new();

        //TODO Task 2: Signature
        let keypair = Keypair::generate(&mut rand::rngs::OsRng {});
        let account_tx = Transaction::new(
            TransactionData::CreateAccount("satoshi".to_string(), keypair.public),
            None,
        );

        assert!(append_block_with_tx(bc, 1, vec![account_tx]).is_ok());

        let mut tx = Transaction::new(
            TransactionData::MintInitialSupply {
                to: "satoshi".to_string(),
                amount: 100_000_000,
            },
            Some("satoshi".to_string()),
        );
        //TODO Task 2: Signature
        tx.add_signature(keypair.sign(tx.hash().as_bytes()).to_bytes());

        assert_eq!(
            append_block_with_tx(bc, 2, vec![tx]).err().unwrap(),
            "Error during executing transactions: Initial Supply can be minted only in genesis block".to_string()
        );
    }

    #[test]
    fn test_initial_supply_works() {
        let mut bc = Blockchain::new();

        let mut block = Block::new(None);
        block.set_nonce(1);
        block.add_transaction(create_account_tx("satoshi".to_string()));
        block.add_transaction(Transaction::new(
            TransactionData::MintInitialSupply {
                to: "satoshi".to_string(),
                amount: 100_000_000,
            },
            None,
        ));
        assert!(bc.append_block(block).is_ok());

        let account = bc.get_account_by_id(&"satoshi".to_string());
        assert!(account.is_some());
        assert_eq!(account.unwrap().balance, 100_000_000);
    }

    #[test]
    fn test_transfer() {
        let bc = &mut Blockchain::new();

        let keypair = Keypair::generate(&mut rand::rngs::OsRng {});
        let account_tx = Transaction::new(
            TransactionData::CreateAccount("satoshi".to_string(), keypair.public),
            None,
        );
        assert!(append_block_with_tx(
            bc,
            1,
            vec![
                account_tx,
                Transaction::new(
                    TransactionData::MintInitialSupply {
                        to: "satoshi".to_string(),
                        amount: 100_000_000,
                    },
                    None,
                )
            ]
        )
        .is_ok());

        let mut tx = Transaction::new(
            TransactionData::Transfer {
                to: "alice".to_string(),
                amount: 10,
            },
            Some("satoshi".to_string()),
        );
        tx.add_signature(keypair.sign(tx.hash().as_bytes()).to_bytes());

        assert!(
            append_block_with_tx(bc, 2, vec![create_account_tx("alice".to_string()), tx]).is_ok()
        );

        let satoshi = bc.get_account_by_id(&"satoshi".to_string());
        let alice = bc.get_account_by_id(&"alice".to_string());
        assert_eq!(alice.unwrap().balance, 10);
        assert_eq!(satoshi.unwrap().balance, 99_999_990);
    }

    #[test]
    fn test_transfer_fails() {
        let bc = &mut Blockchain::new();

        let keypair = Keypair::generate(&mut rand::rngs::OsRng {});
        let tx = Transaction::new(
            TransactionData::CreateAccount("satoshi".to_string(), keypair.public),
            None,
        );

        assert!(append_block_with_tx(
            bc,
            1,
            vec![
                tx,
                Transaction::new(
                    TransactionData::MintInitialSupply {
                        to: "satoshi".to_string(),
                        amount: 100_000_000,
                    },
                    None,
                ),
            ],
        )
        .is_ok());

        let mut tx = Transaction::new(
            TransactionData::Transfer {
                to: "alice".to_string(),
                amount: 100_000_001,
            },
            Some("satoshi".to_string()),
        );
        tx.add_signature(keypair.sign(tx.hash().as_bytes()).to_bytes());
        assert_eq!(
            append_block_with_tx(bc, 2, vec![create_account_tx("alice".to_string()), tx],)
                .err()
                .unwrap(),
            String::from("Error during executing transactions: Insufficient balance")
        );

        let mut tx = Transaction::new(
            TransactionData::Transfer {
                to: "invalid_address".to_string(),
                amount: 10,
            },
            Some("satoshi".to_string()),
        );
        tx.add_signature(keypair.sign(tx.hash().as_bytes()).to_bytes());
        assert_eq!(
            append_block_with_tx(bc, 2, vec![create_account_tx("alice".to_string()), tx],)
                .err()
                .unwrap(),
            String::from("Error during executing transactions: Invalid receiver address.")
        );

        let mut tx = Transaction::new(
            TransactionData::Transfer {
                to: "alice".to_string(),
                amount: 10,
            },
            Some("invalid_address".to_string()),
        );
        tx.add_signature(keypair.sign(tx.hash().as_bytes()).to_bytes());
        assert_eq!(
            append_block_with_tx(bc, 2, vec![create_account_tx("alice".to_string()), tx])
                .err()
                .unwrap(),
            String::from("Error during executing transactions: Account `from` not exist.")
        );
    }

    //TODO Task 2: Signature
    #[test]
    fn test_sign_transaction() {
        let bc = &mut Blockchain::new();

        let keypair = Keypair::generate(&mut rand::rngs::OsRng {});
        let account_tx = Transaction::new(
            TransactionData::CreateAccount("satoshi".to_string(), keypair.public),
            None,
        );

        assert!(append_block_with_tx(
            bc,
            1,
            vec![
                account_tx,
                Transaction::new(
                    TransactionData::MintInitialSupply {
                        to: "satoshi".to_string(),
                        amount: 100_000_000,
                    },
                    Some("satoshi".to_string()),
                ),
            ],
        )
        .is_ok());

        let mut tx = Transaction::new(
            TransactionData::Transfer {
                to: "alice".to_string(),
                amount: 100,
            },
            Some("satoshi".to_string()),
        );
        tx.add_signature(keypair.sign(tx.hash().as_bytes()).to_bytes());

        assert!(
            append_block_with_tx(bc, 2, vec![create_account_tx("alice".to_string()), tx]).is_ok()
        );
    }
}
