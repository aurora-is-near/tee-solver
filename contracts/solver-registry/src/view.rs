use crate::*;
use near_sdk::AccountId;

#[near]
impl Contract {
    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn get_pool_len(&self) -> u32 {
        self.pools.len()
    }

    pub fn get_pool(&self, pool_id: u32) -> Option<PoolInfo> {
        self.pools.get(pool_id).map(|p| PoolInfo {
            token_ids: p.token_ids.clone(),
            amounts: p.amounts.iter().map(|a| (*a).into()).collect(),
            fee: p.fee,
            shares_total_supply: p.shares_total_supply.into(),
            worker_id: p.worker_id.clone(),
            last_ping_timestamp_ms: p.last_ping_timestamp_ms,
        })
    }

    pub fn get_worker_len(&self) -> u32 {
        self.worker_by_account_id.len()
    }

    pub fn get_worker(&self, account_id: AccountId) -> Option<Worker> {
        self.worker_by_account_id.get(&account_id).cloned()
    }

    pub fn get_workers(&self, offset: u32, limit: u32) -> Vec<&Worker> {
        self.worker_by_account_id
            .values()
            .skip(offset as usize)
            .take(limit as usize)
            .collect()
    }

    pub fn get_worker_ping_timeout_ms(&self) -> TimestampMs {
        self.worker_ping_timeout_ms
    }
}
