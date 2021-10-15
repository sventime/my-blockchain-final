use blake2::{Blake2s, Digest};
use ed25519_dalek::{PublicKey, Verifier};

use crate::traits::{Hashable, WorldState};
use crate::types::{AccountId, AccountType, Balance, Error, Hash, Signature, Timestamp};

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionData {
    CreateAccount(AccountId, PublicKey),
    Transfer { to: AccountId, amount: Balance },
    MintInitialSupply { to: AccountId, amount: Balance },
}

#[derive(Debug, Clone)]
pub struct Transaction {
    nonce: u128,
    timestamp: Timestamp,
    pub(crate) data: TransactionData,
    pub(crate) from: Option<AccountId>,
    signature: Option<Signature>,
}

impl Hashable for Transaction {
    fn hash(&self) -> Hash {
        hex::encode(Blake2s::digest(
            format!("{:?}", (self.nonce, self.timestamp, &self.data, &self.from)).as_bytes(),
        ))
    }
}

/// State transition functions

fn create_account<T: WorldState>(
    state: &mut T,
    account_id: AccountId,
    public_key: PublicKey,
) -> Result<(), Error> {
    state.create_account(account_id, AccountType::User, public_key)
}

fn mint_initial_supply<T: WorldState>(
    state: &mut T,
    to: AccountId,
    amount: Balance,
    is_genesis: bool,
) -> Result<(), Error> {
    if !is_genesis {
        return Err("Initial Supply can be minted only in genesis block".to_string());
    }
    match state.get_account_by_id_mut(&to) {
        Some(account) => {
            account.balance += amount;
            Ok(())
        }
        None => Err("Invalid account.".to_string()),
    }
}

// TODO Task 1: Transfer
fn transfer<T: WorldState>(
    state: &mut T,
    from: AccountId,
    to: AccountId,
    amount: Balance,
) -> Result<(), Error> {
    state.get_account_by_id_mut(&from).map_or(
        Err("Invalid sender address.".to_string()),
        |acc| {
            acc.balance.checked_sub(amount).map_or(
                Err("Insufficient balance".to_string()),
                |new_amount| {
                    acc.balance = new_amount;
                    Ok(())
                },
            )
        },
    )?;

    state.get_account_by_id_mut(&to).map_or(
        Err("Invalid receiver address.".to_string()),
        |acc| {
            acc.balance.checked_add(amount).map_or(
                Err("Balance overflow.".to_string()),
                |new_amount| {
                    acc.balance = new_amount;
                    Ok(())
                },
            )
        },
    )?;

    Ok(())
}

impl Transaction {
    pub fn new(data: TransactionData, from: Option<AccountId>) -> Self {
        Self {
            nonce: 0,
            timestamp: 0,
            data,
            from,
            signature: None,
        }
    }

    pub fn set_from(&mut self, from: AccountId) {
        self.from = Some(from)
    }

    //TODO Task 2: Signature
    pub fn add_signature(&mut self, signature: Signature) {
        self.signature = Some(signature);
    }

    pub fn execute<T: WorldState>(&self, state: &mut T, is_genesis: bool) -> Result<(), Error> {
        //TODO Task 2: Signature
        if !is_genesis && !matches!(self.data, TransactionData::CreateAccount(_, _)) {
            if let Err(error) = self.check_signature(state) {
                return Err(error);
            }
        }
        match &self.data {
            TransactionData::CreateAccount(account_id, public_key) => {
                create_account(state, account_id.clone(), *public_key)
            }
            TransactionData::MintInitialSupply { to, amount } => {
                mint_initial_supply(state, to.clone(), *amount, is_genesis)
            }
            TransactionData::Transfer { to, amount } => {
                //TODO Task 1: Transfer
                transfer(state, self.from.clone().unwrap(), to.clone(), *amount)
            }
        }
    }

    fn check_signature<T: WorldState>(&self, state: &mut T) -> Result<(), Error> {
        //TODO Task 2: Signature
        if self.signature.is_none() {
            return Err("Signature is missing.".to_string());
        }
        self.from
            .clone()
            .map_or(Err("Tx `from` is not defined.".to_string()), |from| {
                state.get_account_by_id(&from).map_or(
                    Err("Account `from` not exist.".to_string()),
                    |account| {
                        dbg!(self.hash());
                        if account
                            .public_key
                            .verify(
                                self.hash().as_bytes(),
                                &ed25519_dalek::Signature::from(self.signature.unwrap()),
                            )
                            .is_err()
                        {
                            Err("Invalid signature.".to_string())
                        } else {
                            Ok(())
                        }
                    },
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    use super::*;

    #[test]
    fn test_tx_hash_changed() {
        let keypair = Keypair::generate(&mut OsRng {});
        let mut tx = Transaction::new(
            TransactionData::CreateAccount("alice".to_string(), keypair.public),
            None,
        );
        let hash = tx.hash();
        tx.data = TransactionData::CreateAccount("bob".to_string(), keypair.public);
        let hast_new = tx.hash();

        assert_ne!(hash, hast_new);
    }
}
