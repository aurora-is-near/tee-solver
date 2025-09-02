use near_sdk::json_types::U128;
// use near_sdk::json_types::U128;
use near_sdk::store::LookupMap;
use near_sdk::{near, require, AccountId, Gas, NearToken, PromiseError, PromiseOrValue};

use crate::events::Event;
use crate::ext::ext_ft;
use crate::*;

const CREATE_POOL_STORAGE_DEPOSIT: NearToken =
    NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000); // 1.5 NEAR
const GAS_CREATE_POOL_CALLBACK: Gas = Gas::from_tgas(10);

const ERR_POOL_NOT_FOUND: &str = "Pool not found";
const ERR_BAD_TOKEN_ID: &str = "Token doesn't exist in pool";
const ERR_INVALID_AMOUNT: &str = "Amount must be > 0";

#[near(serializers = [borsh])]
pub struct Pool {
    /// List of tokens in the pool.
    pub token_ids: Vec<AccountId>,
    /// How much NEAR this contract has.
    pub amounts: Vec<Balance>,
    /// Fee charged for swap in basis points
    pub fee: u32,
    /// Shares of the pool by liquidity providers.
    pub shares: LookupMap<AccountId, Balance>,
    /// Total number of shares.
    pub shares_total_supply: Balance,
    /// Worker account ID. Only one worker is allowed per pool.
    pub worker_id: Option<AccountId>,
    /// Last ping timestamp by the pool's worker.
    pub last_ping_timestamp_ms: TimestampMs,
}

#[near(serializers = [json])]
pub struct PoolInfo {
    /// List of tokens in the pool.
    pub token_ids: Vec<AccountId>,
    /// How much NEAR this contract has.
    pub amounts: Vec<U128>,
    /// Fee charged for swap in basis points
    pub fee: u32,
    /// Total number of shares.
    pub shares_total_supply: U128,
    /// Worker account ID. Only one worker is allowed per pool.
    pub worker_id: Option<AccountId>,
    /// Last ping timestamp by the pool's worker.
    pub last_ping_timestamp_ms: TimestampMs,
}

impl Pool {
    pub fn new(token_ids: Vec<AccountId>, fee: u32) -> Self {
        require!(token_ids.len() == 2, "Must have exactly 2 tokens");
        require!(
            token_ids[0] != token_ids[1],
            "The two tokens cannot be identical"
        );
        require!(fee < 10_000, "Fee must be less than 100%");

        Self {
            token_ids: token_ids.clone(),
            amounts: vec![0; token_ids.len()],
            fee,
            shares: LookupMap::new(Prefix::PoolShares),
            shares_total_supply: 0,
            worker_id: None,
            last_ping_timestamp_ms: 0,
        }
    }

    /// Assume the worker is active if there's a ping within the timeout period.
    pub fn has_active_worker(&self, timeout_ms: TimestampMs) -> bool {
        self.worker_id.is_some() && block_timestamp_ms() < self.last_ping_timestamp_ms + timeout_ms
    }
}

#[near]
impl Contract {
    /// Create a new liquidity pool for the given NEP-141 token IDs with fee in basis points
    #[payable]
    pub fn create_liquidity_pool(
        &mut self,
        token_ids: Vec<AccountId>,
        fee: u32,
    ) -> PromiseOrValue<Option<u32>> {
        require!(
            env::attached_deposit() >= CREATE_POOL_STORAGE_DEPOSIT,
            "Not enough attached deposit"
        );

        // Get new pool ID
        let pool_id = self.pools.len();

        // Create sub account for managing liquidity pool's assets in NEAR Intents
        let pool_account_id = self.get_pool_account_id(pool_id);
        Promise::new(pool_account_id)
            .create_account()
            .transfer(CREATE_POOL_STORAGE_DEPOSIT)
            .deploy_contract(include_bytes!("../../intents-vault/res/intents_vault.wasm").to_vec())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_CREATE_POOL_CALLBACK)
                    .on_create_liquidity_pool_account(pool_id, token_ids, fee),
            )
            .into()
    }

    #[private]
    pub fn on_create_liquidity_pool_account(
        &mut self,
        pool_id: u32,
        token_ids: Vec<AccountId>,
        fee: u32,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Option<u32> {
        if call_result.is_err() {
            None
        } else {
            // Add the new liquidity pool
            let pool = Pool::new(token_ids.clone(), fee);
            self.pools.push(pool);
            self.pools.flush();

            Event::CreateLiquidityPool {
                pool_id: &pool_id,
                token_ids: &token_ids,
                fee: &fee,
            }
            .emit();

            Some(pool_id)
        }
    }

    #[private]
    pub fn on_deposit_into_pool(
        &mut self,
        amount: U128,
        #[callback_result] used_fund: Result<U128, PromiseError>,
    ) -> U128 {
        if let Ok(used_fund) = used_fund {
            // Refund the unused amount.
            // ft_transfser_call() returns the used fund
            U128(amount.0.saturating_sub(used_fund.0))
        } else {
            amount
        }
    }
}

impl Contract {
    pub(crate) fn get_pool_account_id(&self, pool_id: u32) -> AccountId {
        format!("pool-{}.{}", pool_id, env::current_account_id())
            .parse()
            .unwrap()
    }

    pub(crate) fn deposit_into_pool(
        &self,
        pool_id: u32,
        token_id: &AccountId,
        _sender_id: &AccountId,
        amount: Balance,
    ) -> PromiseOrValue<U128> {
        let pool = self.pools.get(pool_id).expect(ERR_POOL_NOT_FOUND);

        require!(pool.token_ids.contains(token_id), ERR_BAD_TOKEN_ID);
        require!(amount > 0, ERR_INVALID_AMOUNT);

        // deposit the fund into NEAR Intents
        // NEAR Intents docs: https://docs.near-intents.org/near-intents/market-makers/verifier/deposits-and-withdrawals/deposits
        ext_ft::ext(token_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer_call(
                self.intents_contract_id.clone(),
                U128(amount),
                Some("deposit into pool".to_string()),
                self.get_pool_account_id(pool_id).to_string(),
            )
            .then(Self::ext(env::current_account_id()).on_deposit_into_pool(U128(amount)))
            .into()
    }
}
