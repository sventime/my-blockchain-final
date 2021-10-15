use crate::types::{AccountId, Balance, Transaction, TransactionData};
use blake2::{Blake2s, Digest};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use rand::Rng;

pub fn create_mint_initial_supply_tx(to: AccountId, amount: Balance) -> TransactionData {
    TransactionData::MintInitialSupply { to, amount }
}

pub fn create_account_tx(account_id: String) -> Transaction {
    let keypair = Keypair::generate(&mut OsRng {});
    Transaction::new(
        TransactionData::CreateAccount(account_id, keypair.public),
        None,
    )
}

pub fn generate_random_account() -> AccountId {
    let mut rng = rand::thread_rng();
    let seed: u128 = rng.gen();

    hex::encode(Blake2s::digest(&seed.to_be_bytes()))
}
