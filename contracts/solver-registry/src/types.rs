use near_sdk::{BorshStorageKey, near};

pub type Balance = u128;
pub type TimestampMs = u64;

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    Pools,
    PoolShares,
    ApprovedComposeHashes,
    WorkerByAccountId,
}
