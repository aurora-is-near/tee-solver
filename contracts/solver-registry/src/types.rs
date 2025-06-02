use near_sdk::{near, BorshStorageKey};

pub type Balance = u128;

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    Pools,
    PoolShares,
    ApprovedCodeHashes,
    WorkerByAccountId,
}
