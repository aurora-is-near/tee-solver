use crate::*;
use near_sdk::{json_types::U128, AccountId};

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
            fees_per_share: p.fees_per_share.iter().map(|f| (*f).into()).collect(),
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

    /// Get pool information
    pub fn get_pool_info(&self, pool_id: u32) -> Option<PoolInfo> {
        self.pools.get(pool_id).map(|pool| PoolInfo {
            token_ids: pool.token_ids.clone(),
            amounts: pool.amounts.iter().map(|&a| a.into()).collect(),
            fee: pool.fee,
            shares_total_supply: pool.shares_total_supply.into(),
            fees_per_share: pool.fees_per_share.iter().map(|&f| f.into()).collect(),
        })
    }

    /// Get liquidity provider shares for a specific pool
    pub fn get_liquidity_provider_shares(&self, pool_id: u32, account_id: AccountId) -> U128 {
        if let Some(pool) = self.pools.get(pool_id) {
            U128(*pool.shares.get(&account_id).unwrap_or(&0u128))
        } else {
            U128(0)
        }
    }

    /// Get pending rewards for a liquidity provider
    pub fn get_pending_rewards(&self, pool_id: u32, account_id: AccountId) -> Vec<U128> {
        if let Some(pool) = self.pools.get(pool_id) {
            pool.calculate_pending_rewards(&account_id)
                .into_iter()
                .map(|r| U128(r))
                .collect()
        } else {
            vec![U128(0), U128(0)]
        }
    }
}
