use near_sdk::json_types::U128;
use near_sdk::store::LookupMap;
use near_sdk::{near, require, AccountId, NearToken};

use crate::*;

const CREATE_POOL_STORAGE_DEPOSIT: NearToken = NearToken::from_near(1);

#[near(serializers = [borsh])]
pub struct Pool {
    /// List of tokens in the pool.
    pub token_ids: Vec<AccountId>,
    /// How much NEAR this contract has.
    pub amounts: Vec<Balance>,
    /// Fee charged for swap in basis points
    pub fee: u64,
    /// Shares of the pool by liquidity providers.
    pub shares: LookupMap<AccountId, Balance>,
    /// Total number of shares.
    pub shares_total_supply: Balance,
}

impl Pool {
    pub fn new(token_ids: Vec<AccountId>, fee: u64) -> Self {
        require!(token_ids.len() == 2, "Must have exactly 2 tokens");
        require!(fee < 10_000, "Fee must be less than 100%");

        Self {
            token_ids: token_ids.clone(),
            amounts: vec![0; token_ids.len()],
            fee,
            shares: LookupMap::new(Prefix::PoolShares),
            shares_total_supply: 0,
        }
    }
}

#[near]
impl Contract {
    #[payable]
    pub fn create_liquidity_pool(&mut self, token_ids: Vec<AccountId>, fee: u64) -> u32 {
        require!(token_ids.len() == 2, "Must have exactly 2 tokens");
        require!(fee <= 10000, "Fee must be less than or equal to 100%");

        let pool_id = self.pools.len();
        let pool = Pool::new(token_ids, fee);
        self.pools.push(pool);
        self.pools.flush();

        // TODO: create sub account for managing intents assets

        let pool_account_id = self.get_pool_account_id(pool_id);
        Promise::new(pool_account_id.clone())
            .create_account()
            .transfer(CREATE_POOL_STORAGE_DEPOSIT)
            .deploy_contract(include_bytes!("../../intents-vault/res/intents_vault.wasm").to_vec());

        pool_id
    }

    #[payable]
    pub fn add_liquidity(
        &mut self,
        pool_id: u32,
        token_ids: Vec<AccountId>,
        amounts: Vec<Balance>,
    ) {
        require!(token_ids.len() == 2, "Must have exactly 2 tokens");
        require!(amounts.len() == 2, "Must have exactly 2 amounts");
        require!(amounts[0] > 0, "Amount must be greater than 0");
        require!(amounts[1] > 0, "Amount must be greater than 0");

        let pool = self.pools.get(pool_id).expect("Pool not found");
        let shares_total_supply = pool.shares_total_supply;
    }

    #[payable]
    pub fn remove_liquidity(&mut self, pool_id: u32, shares: U128) {
        let shares = shares.0;
        require!(shares > 0, "Shares must be greater than 0");

        let pool = self.pools.get(pool_id).expect("Pool not found");
        // pool.shares_total_supply -= shares;
    }
}

impl Contract {
    pub(crate) fn get_pool_account_id(&self, pool_id: u32) -> AccountId {
        format!("pool-{}.{}", pool_id, env::current_account_id()).parse().unwrap()
    }

    pub(crate) fn has_pool(&self, pool_id: u32) -> bool {
        self.pools.get(pool_id).is_some()
    }
}
