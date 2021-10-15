mod account;
mod block;
mod blockchain;
mod chain;
mod transaction;

pub use self::blockchain::Blockchain;
pub use account::{Account, AccountType};
pub use block::Block;
pub use transaction::{Transaction, TransactionData};

pub type AccountId = String;
pub type Balance = u128;
/// Millist sinse unix epoch
pub type Timestamp = u128;
pub type Hash = String;
pub type Signature = [u8; 64];
pub type Error = String;
