use near_sdk::{near, BorshStorageKey};

pub type Balance = u128;
pub type TimestampMs = u64;

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    Pools,
    PoolShares,
    ApprovedCodeHashes,
    WorkerByAccountId,
}
