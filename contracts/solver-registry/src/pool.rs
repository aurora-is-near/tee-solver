use near_sdk::json_types::U128;
use near_sdk::store::LookupMap;
use near_sdk::{
    assert_one_yocto, env, near, require, AccountId, Gas, NearToken, Promise, PromiseError,
    PromiseOrValue,
};

use crate::events::Event;
use crate::ext::ext_ft;
use crate::nep245::ext_mt_core;
use crate::*;

const CREATE_POOL_STORAGE_DEPOSIT: NearToken =
    NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000); // 1.5 NEAR
const GAS_CREATE_POOL_CALLBACK: Gas = Gas::from_tgas(10);
const GAS_REMOVE_LIQUIDITY_CALLBACK: Gas = Gas::from_tgas(10);
const GAS_CLAIM_REWARDS_CALLBACK: Gas = Gas::from_tgas(10);
const GAS_SYNC_BALANCES_FROM_INTENTS: Gas = Gas::from_tgas(5);

const ERR_POOL_NOT_FOUND: &str = "Pool not found";
const ERR_BAD_TOKEN_ID: &str = "Token doesn't exist in pool";
const ERR_INVALID_AMOUNT: &str = "Amount must be > 0";
const ERR_INSUFFICIENT_SHARES: &str = "Insufficient shares";

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
    /// Unclaimed fees for each token
    pub unclaimed_fees: Vec<Balance>,
    /// Accumulated fees per share for each token (for reward calculation)
    pub fees_per_share: Vec<Balance>,
    /// Last claimed fees per share for each liquidity provider
    pub last_claimed_fees: LookupMap<AccountId, Vec<Balance>>,
}

#[near(serializers = [json])]
pub struct PoolInfo {
    /// List of tokens in the pool.
    pub token_ids: Vec<AccountId>,
    /// How much NEAR this contract has.
    pub amounts: Vec<U128>,
    /// Fee charged for swap in basis points
    pub fee: u32,
    /// Unclaimed fees for each token
    pub unclaimed_fees: Vec<U128>,
    /// Total number of shares.
    pub shares_total_supply: U128,
    /// Accumulated fees per share for each token
    pub fees_per_share: Vec<U128>,
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
            unclaimed_fees: vec![0; token_ids.len()],
            fees_per_share: vec![0; token_ids.len()],
            last_claimed_fees: LookupMap::new(Prefix::LastClaimedFees),
        }
    }

    pub fn get_token_ids(&self) -> Vec<AccountId> {
        self.token_ids.clone()
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(&mut self, account_id: &AccountId, amounts: Vec<Balance>) -> Balance {
        require!(amounts.len() == 2, "Must have exactly 2 amounts");
        require!(
            amounts[0] > 0 && amounts[1] > 0,
            "Amounts must be greater than 0"
        );

        // Calculate shares to mint
        let shares_to_mint = self.calculate_shares(&amounts);
        require!(shares_to_mint > 0, "Invalid share calculation");

        // Update pool amounts
        for (i, amount) in amounts.iter().enumerate() {
            self.amounts[i] += amount;
        }

        // Mint shares to liquidity provider
        let current_shares = self.shares.get(account_id).unwrap_or(&0u128);
        self.shares
            .insert(account_id.clone(), current_shares + shares_to_mint);
        self.shares_total_supply += shares_to_mint;

        shares_to_mint
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(
        &mut self,
        account_id: &AccountId,
        shares_to_burn: Balance,
    ) -> Vec<Balance> {
        require!(shares_to_burn > 0, "Shares must be greater than 0");

        let current_shares = self.shares.get(account_id).unwrap_or(&0u128);
        require!(current_shares >= &shares_to_burn, ERR_INSUFFICIENT_SHARES);

        // Calculate amounts to return based on share proportion
        let mut amounts_to_return = Vec::new();
        for amount in &self.amounts {
            let amount_to_return = (amount * shares_to_burn) / self.shares_total_supply;
            amounts_to_return.push(amount_to_return);
        }

        // Update pool amounts
        for (i, amount) in amounts_to_return.iter().enumerate() {
            self.amounts[i] -= amount;
        }

        // Burn shares
        let new_shares = current_shares - shares_to_burn;
        if new_shares == 0 {
            self.shares.remove(account_id);
        } else {
            self.shares.insert(account_id.clone(), new_shares);
        }
        self.shares_total_supply -= shares_to_burn;

        amounts_to_return
    }

    /// Calculate pending rewards for a liquidity provider
    pub fn calculate_pending_rewards(&self, account_id: &AccountId) -> Vec<Balance> {
        let shares = self.shares.get(account_id).unwrap_or(&0u128);
        if shares == &0u128 {
            return vec![0; self.token_ids.len()];
        }

        let last_claimed = self.last_claimed_fees.get(account_id);
        let mut pending_rewards = Vec::new();

        for (i, fees_per_share) in self.fees_per_share.iter().enumerate() {
            let last_claimed_fee = last_claimed
                .as_ref()
                .and_then(|claimed| claimed.get(i))
                .unwrap_or(&0u128);
            let pending = (fees_per_share - last_claimed_fee) * shares;
            pending_rewards.push(pending);
        }

        pending_rewards
    }

    /// Update fees per share (called after collecting fees)
    pub fn update_fees_in_pool(&mut self, collected_fees: Vec<Balance>) {
        if self.shares_total_supply > 0 {
            for (i, fee) in collected_fees.iter().enumerate() {
                self.unclaimed_fees[i] += fee;
                self.fees_per_share[i] += fee / self.shares_total_supply;
            }
        }
    }

    /// Mark fees as claimed for a liquidity provider
    pub fn mark_fees_claimed(&mut self, account_id: &AccountId) {
        let pending_rewards = self.calculate_pending_rewards(account_id);
        for (i, fee) in pending_rewards.iter().enumerate() {
            self.unclaimed_fees[i] -= *fee;
        }
        self.last_claimed_fees
            .insert(account_id.clone(), self.fees_per_share.clone());
    }

    /// Calculate shares to mint based on deposited amounts
    fn calculate_shares(&self, amounts: &[Balance]) -> Balance {
        if self.shares_total_supply == 0 {
            // First liquidity provider gets shares equal to geometric mean of amounts
            Self::sqrt(amounts[0] * amounts[1])
        } else {
            // Calculate shares based on current pool ratios
            let share0 = (amounts[0] * self.shares_total_supply) / self.amounts[0];
            let share1 = (amounts[1] * self.shares_total_supply) / self.amounts[1];
            if share0 < share1 {
                share0
            } else {
                share1
            }
        }
    }

    /// Calculate square root for u128
    fn sqrt(value: Balance) -> Balance {
        if value == 0 {
            return 0;
        }
        let mut x = value;
        let mut y = x.div_ceil(2);
        while y < x {
            x = y;
            y = (x + value / x) / 2;
        }
        x
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
        Promise::new(pool_account_id.clone())
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

    /// Add liquidity to a pool
    /// Users must call ft_transfer_call for each token before calling this function
    #[payable]
    pub fn add_liquidity(&mut self, pool_id: u32, amounts: Vec<U128>) -> PromiseOrValue<U128> {
        assert_one_yocto();

        let amounts: Vec<Balance> = amounts.into_iter().map(|a| a.0).collect();
        require!(amounts.len() == 2, "Must have exactly 2 amounts");
        require!(amounts[0] > 0 && amounts[1] > 0, ERR_INVALID_AMOUNT);

        let account_id = env::predecessor_account_id();

        self.sync_balances_from_intents(pool_id)
            .then(Self::ext(env::current_account_id()).on_add_liquidity_start(
                account_id,
                pool_id,
                amounts.iter().map(|x| U128(*x)).collect(),
            ))
            .into()
    }

    /// Start adding liquidity to a pool
    /// This function is called after syncing balances from NEAR Intents
    /// It withdraws funds from account deposits and adds liquidity to the pool
    /// It returns the number of shares minted
    #[private]
    pub fn on_add_liquidity_start(
        &mut self,
        account_id: AccountId,
        pool_id: u32,
        amounts: Vec<U128>,
        #[callback_result] call_result: Result<Vec<U128>, PromiseError>,
    ) -> PromiseOrValue<U128> {
        if call_result.is_err() {
            env::panic_str("Failed to sync balances from NEAR Intents");
        }
        let amounts: Vec<Balance> = amounts.into_iter().map(|a| a.0).collect();

        // Withdraw funds from accounts first
        let token_ids = self.withdraw_from_accounts(pool_id, &account_id, &amounts);

        // Add liquidity to the pool
        let pool = self.pools.get_mut(pool_id).expect(ERR_POOL_NOT_FOUND);
        let shares_to_mint = pool.add_liquidity(&account_id, amounts.clone());

        // transfer tokens to NEAR Intents account
        ext_ft::ext(token_ids[0].clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer_call(
                self.intents_contract_id.clone(),
                U128(amounts[0]),
                Some("deposit into pool".to_string()),
                self.get_pool_account_id(pool_id).to_string(),
            )
            .and(
                ext_ft::ext(token_ids[1].clone())
                    .with_attached_deposit(NearToken::from_yoctonear(1))
                    .ft_transfer_call(
                        self.intents_contract_id.clone(),
                        U128(amounts[1]),
                        Some("deposit into pool".to_string()),
                        self.get_pool_account_id(pool_id).to_string(),
                    ),
            )
            .then(Self::ext(env::current_account_id()).on_add_liquidity_end(
                pool_id,
                amounts.iter().map(|x| U128(*x)).collect(),
                account_id,
                U128(shares_to_mint),
            ))
            .into()
    }

    #[private]
    pub fn on_add_liquidity_end(
        &self,
        pool_id: u32,
        amounts: Vec<U128>,
        account_id: AccountId,
        shares_to_mint: U128,
        #[callback_vec] used_fund: Vec<Result<U128, PromiseError>>,
    ) -> U128 {
        if used_fund.iter().all(|x| x.is_ok()) {
            Event::AddLiquidity {
                pool_id: &pool_id,
                account_id: &account_id,
                amounts: &amounts.into_iter().collect(),
                shares_minted: &shares_to_mint,
            }
            .emit();

            shares_to_mint
        } else {
            // TODO: rollback the minted shares
            // The failed transfer should be kept in `lost_and_found` and can be withdrawn by the user

            shares_to_mint
        }
    }

    /// Remove liquidity from a pool
    #[payable]
    pub fn remove_liquidity(&mut self, pool_id: u32, shares: U128) -> PromiseOrValue<Vec<U128>> {
        assert_one_yocto();

        let shares_to_burn = shares.0;
        require!(shares_to_burn > 0, "Shares must be greater than 0");

        let account_id = env::predecessor_account_id();
        self.sync_balances_from_intents(pool_id)
            .then(
                Self::ext(env::current_account_id())
                    .on_remove_liquidity_start(account_id, pool_id, shares),
            )
            .into()
    }

    #[private]
    pub fn on_remove_liquidity_start(
        &mut self,
        account_id: AccountId,
        pool_id: u32,
        shares: U128,
    ) -> PromiseOrValue<Vec<U128>> {
        let pool = self.pools.get_mut(pool_id).expect(ERR_POOL_NOT_FOUND);
        let token_ids = pool.token_ids.clone();
        let shares_to_burn = shares.0;

        // Remove liquidity and get amounts to return
        let amounts_to_return = pool.remove_liquidity(&account_id, shares_to_burn);
        self.pools.flush();

        // TODO: ft_withdraw with static gas
        // TODO: handle cases when user is not a NEAR account
        let pool_account_id = self.get_pool_account_id(pool_id);
        ext_intents_vault::ext(pool_account_id.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_withdraw(
                self.intents_contract_id.clone(),
                token_ids[0].clone(),
                account_id.clone(),
                U128(amounts_to_return[0]),
                None,
                None,
            )
            .and(
                ext_intents_vault::ext(pool_account_id.clone())
                    .with_attached_deposit(NearToken::from_yoctonear(1))
                    .ft_withdraw(
                        self.intents_contract_id.clone(),
                        token_ids[1].clone(),
                        account_id.clone(),
                        U128(amounts_to_return[1]),
                        None,
                        None,
                    ),
            )
            .then(
                Self::ext(env::current_account_id()).on_remove_liquidity_end(
                    pool_id,
                    account_id.clone(),
                    amounts_to_return.into_iter().map(U128).collect(),
                    U128(shares_to_burn),
                ),
            )
            .into()
    }

    #[private]
    pub fn on_remove_liquidity_end(
        &self,
        pool_id: u32,
        account_id: AccountId,
        amounts: Vec<U128>,
        shares: U128,
        #[callback_vec] results: Vec<Result<U128, PromiseError>>,
    ) -> Vec<U128> {
        if results.iter().all(|x| x.is_ok()) {
            Event::RemoveLiquidity {
                pool_id: &pool_id,
                account_id: &account_id,
                amounts: &amounts,
                shares_burned: &shares,
            }
            .emit();

            amounts
        } else {
            // TODO: rollback the removed liquidity
            // The failed transfer should be kept in `lost_and_found` and can be withdrawn by the user
            amounts
        }
    }

    /// Collect fees for a liquidity pool
    /// This function is called when a worker swaps tokens
    /// The fees are recorded to the liquidity pool contract
    /// and can be claimed by the liquidity provider
    /// TODO: whether transfer fees to the liquidity pool contract when collecting fees?
    #[payable]
    pub fn collect_pool_fees(&mut self, fees: Vec<Balance>) {
        let worker = self.require_approved_worker();
        let pool_id = worker.pool_id;

        let pool = self.pools.get_mut(pool_id).expect(ERR_POOL_NOT_FOUND);
        pool.update_fees_in_pool(fees);
    }

    /// Claim accumulated rewards for a liquidity provider
    #[payable]
    pub fn claim_rewards(&mut self, pool_id: u32) -> PromiseOrValue<Vec<U128>> {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();
        let pool = self.pools.get_mut(pool_id).expect(ERR_POOL_NOT_FOUND);
        let token_ids = pool.token_ids.clone();

        // Calculate pending rewards
        let pending_rewards = pool.calculate_pending_rewards(&account_id);
        require!(
            pending_rewards[0] > 0 || pending_rewards[1] > 0,
            "No rewards to claim"
        );

        // Mark fees as claimed
        pool.mark_fees_claimed(&account_id);
        // self.pools.flush();
        // TODO: update pool

        // Transfer rewards to the user
        let pool_account_id = self.get_pool_account_id(pool_id);
        let transfer_promises = if pending_rewards[0] > 0 && pending_rewards[1] > 0 {
            ext_intents_vault::ext(pool_account_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_withdraw(
                    self.intents_contract_id.clone(),
                    token_ids[0].clone(),
                    account_id.clone(),
                    U128(pending_rewards[0]),
                    None,
                    None,
                )
                .and(
                    ext_intents_vault::ext(pool_account_id.clone())
                        .with_attached_deposit(NearToken::from_yoctonear(1))
                        .ft_withdraw(
                            self.intents_contract_id.clone(),
                            token_ids[1].clone(),
                            account_id.clone(),
                            U128(pending_rewards[1]),
                            None,
                            None,
                        ),
                )
        } else if pending_rewards[0] > 0 {
            ext_intents_vault::ext(pool_account_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_withdraw(
                    self.intents_contract_id.clone(),
                    token_ids[0].clone(),
                    account_id.clone(),
                    U128(pending_rewards[0]),
                    None,
                    None,
                )
        } else if pending_rewards[1] > 0 {
            ext_intents_vault::ext(pool_account_id.clone())
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .ft_withdraw(
                    self.intents_contract_id.clone(),
                    token_ids[1].clone(),
                    account_id.clone(),
                    U128(pending_rewards[1]),
                    None,
                    None,
                )
        } else {
            env::panic_str("No rewards to claim");
        };

        transfer_promises
            .then(Self::ext(env::current_account_id()).on_claim_rewards(
                pool_id,
                account_id.clone(),
                pending_rewards.into_iter().map(U128).collect(),
            ))
            .into()
    }

    #[private]
    pub fn on_claim_rewards(
        &self,
        pool_id: u32,
        account_id: AccountId,
        rewards: Vec<U128>,
        #[callback_vec] results: Vec<Result<U128, PromiseError>>,
    ) -> Vec<U128> {
        if results.iter().all(|x| x.is_ok()) {
            Event::ClaimRewards {
                pool_id: &pool_id,
                account_id: &account_id,
                rewards: &rewards,
            }
            .emit();

            rewards
        } else {
            // TODO: rollback the claimed rewards
            // The failed transfer should be kept in `lost_and_found` and can be withdrawn by the user
            rewards
        }
    }

    /// Sync balances from NEAR Intents
    pub fn sync_balances_from_intents(&self, pool_id: u32) -> Promise {
        let pool = self.internal_get_pool(pool_id);
        let token_ids = pool
            .token_ids
            .iter()
            .map(|id| format!("nep141:{}", id))
            .collect();

        ext_mt_core::ext(self.intents_contract_id.clone())
            .with_static_gas(GAS_SYNC_BALANCES_FROM_INTENTS)
            .mt_batch_balance_of(self.get_pool_account_id(pool_id).clone(), token_ids)
            .then(Self::ext(env::current_account_id()).on_sync_balances_from_intents(pool_id))
    }

    #[private]
    pub fn on_sync_balances_from_intents(
        &mut self,
        pool_id: u32,
        #[callback_result] call_result: Result<Vec<U128>, PromiseError>,
    ) {
        if call_result.is_err() {
            env::panic_str("Failed to sync balances from NEAR Intents");
        }
        let balances = call_result.unwrap();
        let pool = self.pools.get_mut(pool_id).expect(ERR_POOL_NOT_FOUND);
        for (i, balance) in balances.iter().enumerate() {
            pool.amounts[i] = balance.0.saturating_sub(pool.unclaimed_fees[i]);
        }
        self.pools.flush();
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

    pub(crate) fn has_pool(&self, pool_id: u32) -> bool {
        self.pools.get(pool_id).is_some()
    }

    pub(crate) fn internal_get_pool(&self, pool_id: u32) -> &Pool {
        self.pools.get(pool_id).expect(ERR_POOL_NOT_FOUND)
    }

    pub(crate) fn deposit_into_pool(
        &self,
        pool_id: u32,
        token_id: &AccountId,
        amount: Balance,
    ) -> PromiseOrValue<U128> {
        let pool = self.internal_get_pool(pool_id);

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

    /// Withdraw balances from accounts and add to liquidity pool
    fn withdraw_from_accounts(
        &mut self,
        pool_id: u32,
        account_id: &AccountId,
        amounts: &[Balance],
    ) -> Vec<AccountId> {
        let token_ids = self.internal_get_pool(pool_id).get_token_ids();

        for (i, token_id) in token_ids.iter().enumerate() {
            let amount = amounts[i];
            self.withdraw_from_account(account_id.clone(), token_id.clone(), amount);
        }

        token_ids
    }
}
