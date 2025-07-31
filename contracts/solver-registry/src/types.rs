use near_sdk::{near, AccountId, BorshStorageKey};

pub type Balance = u128;

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    ApprovedCodeHashes,
    WorkerByAccountId,
    Accounts,
    AccountDeposits { account_id: AccountId },
    Pools,
    PoolShares,
    LpClaimedFeesPerShares,
    LpWithdrawableFees,
}
