use crate::types::{Account, AccountId, AccountType, Hash};
use ed25519_dalek::PublicKey;

pub trait WorldState {
    fn get_account_ids(&self) -> Vec<AccountId>;
    fn get_account_by_id(&self, id: &AccountId) -> Option<&Account>;
    fn get_account_by_id_mut(&mut self, id: &AccountId) -> Option<&mut Account>;
    fn create_account(
        &mut self,
        account_id: AccountId,
        account_type: AccountType,
        public_key: PublicKey,
    ) -> Result<(), String>;
}

pub trait Hashable {
    fn hash(&self) -> Hash;
}

pub trait Verifiable: Hashable {
    fn verify(&self) -> bool;
}
