use near_sdk::{near, AccountId, BorshStorageKey};

pub type Balance = u128;

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    Pools,
    PoolShares,
    LastClaimedFees,
    ApprovedCodeHashes,
    WorkerByAccountId,
    Accounts,
    AccountDeposits { account_id: AccountId },
}
