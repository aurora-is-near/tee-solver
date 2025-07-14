use crate::nep245::interface::events::{MtEvent, MtTransferEvent};
use crate::nep245::interface::{ext_mt_receiver, MultiTokenCore, Token, TokenId};
use crate::*;
use near_sdk::{json_types::U128, PromiseOrValue};
use std::borrow::Cow;

const MT_RESOLVE_GAS: Gas = Gas::from_tgas(10);

#[near]
impl MultiTokenCore for Contract {
    #[payable]
    fn mt_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        amount: U128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
    ) {
        self.mt_batch_transfer(
            receiver_id,
            vec![token_id],
            vec![amount],
            approval.map(|a| vec![Some(a)]),
            memo,
        );
    }

    #[payable]
    fn mt_batch_transfer(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
    ) {
        require!(approvals.is_none(), "approvals are not supported");

        self.internal_mt_batch_transfer(
            &env::predecessor_account_id(),
            &receiver_id,
            token_ids,
            amounts,
            memo,
        );
    }

    #[payable]
    fn mt_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        amount: U128,
        approval: Option<(AccountId, u64)>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        self.mt_batch_transfer_call(
            receiver_id,
            vec![token_id],
            vec![amount],
            approval.map(|a| vec![Some(a)]),
            memo,
            msg,
        )
    }

    #[payable]
    fn mt_batch_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        approvals: Option<Vec<Option<(AccountId, u64)>>>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        self.internal_mt_batch_transfer_call(
            &env::predecessor_account_id(),
            &receiver_id,
            token_ids,
            amounts,
            memo,
            msg,
        )
    }

    fn mt_token(&self, token_ids: Vec<TokenId>) -> Vec<Option<Token>> {
        token_ids
            .into_iter()
            .map(|token_id| {
                // Check if this is a valid pool share token
                if let Some(pool_id) = self.get_pool_id_from_token_id(&token_id) {
                    if self.pools.get(pool_id).is_some() {
                        return Some(Token {
                            token_id,
                            owner_id: None,
                        });
                    }
                }
                None
            })
            .collect()
    }

    fn mt_balance_of(&self, account_id: AccountId, token_id: TokenId) -> U128 {
        U128(self.internal_mt_balance_of(&account_id, &token_id))
    }

    fn mt_batch_balance_of(&self, account_id: AccountId, token_ids: Vec<TokenId>) -> Vec<U128> {
        token_ids
            .into_iter()
            .map(|token_id| self.internal_mt_balance_of(&account_id, &token_id))
            .map(U128)
            .collect()
    }

    fn mt_supply(&self, token_id: TokenId) -> Option<U128> {
        if let Some(pool_id) = self.get_pool_id_from_token_id(&token_id) {
            if let Some(pool) = self.pools.get(pool_id) {
                return Some(U128(pool.shares_total_supply));
            }
        }
        None
    }

    fn mt_batch_supply(&self, token_ids: Vec<TokenId>) -> Vec<Option<U128>> {
        token_ids
            .into_iter()
            .map(|token_id| self.mt_supply(token_id))
            .collect()
    }
}

impl Contract {
    // NEP-245 helper methods using existing pool structure
    pub(crate) fn internal_mt_balance_of(&self, account_id: &AccountId, token_id: &String) -> u128 {
        let pool_id = self.get_pool_id_from_token_id(token_id).unwrap();
        let pool = self.internal_get_pool(pool_id);
        pool.shares.get(account_id).copied().unwrap_or(0)
    }

    pub(crate) fn internal_mt_mint(
        &mut self,
        receiver_id: &AccountId,
        token_id: &String,
        amount: u128,
    ) {
        let pool_id = self.get_pool_id_from_token_id(token_id).unwrap();
        let pool = self.pools.get_mut(pool_id).unwrap();
        let current_shares = pool.shares.get(receiver_id).copied().unwrap_or(0);
        pool.shares
            .insert(receiver_id.clone(), current_shares + amount);
        pool.shares_total_supply += amount;
    }

    pub(crate) fn internal_mt_burn(
        &mut self,
        owner_id: &AccountId,
        token_id: &String,
        amount: u128,
    ) {
        let pool_id = self.get_pool_id_from_token_id(token_id).unwrap();
        let pool = self.pools.get_mut(pool_id).unwrap();
        let current_shares = pool.shares.get(owner_id).copied().unwrap_or(0);
        require!(current_shares >= amount, "Insufficient balance to burn");

        let new_shares = current_shares - amount;
        if new_shares == 0 {
            pool.shares.remove(owner_id);
        } else {
            pool.shares.insert(owner_id.clone(), new_shares);
        }
        pool.shares_total_supply -= amount;
    }

    pub(crate) fn internal_mt_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &String,
        amount: u128,
    ) {
        require!(amount > 0, "Amount must be greater than 0");

        let pool_id = self.get_pool_id_from_token_id(token_id).unwrap();
        let pool = self.pools.get_mut(pool_id).unwrap();
        let sender_shares = pool.shares.get(sender_id).copied().unwrap_or(0);
        require!(sender_shares >= amount, "Insufficient balance");

        let new_sender_shares = sender_shares - amount;
        pool.shares.insert(sender_id.clone(), new_sender_shares);

        let receiver_shares = pool.shares.get(receiver_id).copied().unwrap_or(0);
        pool.shares
            .insert(receiver_id.clone(), receiver_shares + amount);
    }

    pub(crate) fn internal_mt_batch_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
    ) {
        assert_one_yocto();

        require!(sender_id != receiver_id, "Cannot transfer to self");
        require!(
            token_ids.len() == amounts.len(),
            "Token IDs and amounts must have same length"
        );
        require!(!token_ids.is_empty(), "Must transfer at least one token");

        // Perform the transfers
        for (token_id, amount) in token_ids.iter().zip(amounts.iter()) {
            self.internal_mt_transfer(sender_id, receiver_id, token_id, amount.0);
        }

        // Emit transfer event
        MtEvent::MtTransfer(
            [MtTransferEvent {
                authorized_id: None,
                old_owner_id: Cow::Owned(sender_id.clone()),
                new_owner_id: Cow::Owned(receiver_id.clone()),
                token_ids: Cow::Owned(token_ids),
                amounts: Cow::Owned(amounts),
                memo: memo.map(Cow::Owned),
            }]
            .as_slice()
            .into(),
        )
        .emit();
    }

    pub(crate) fn internal_mt_batch_transfer_call(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_ids: Vec<TokenId>,
        amounts: Vec<U128>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<Vec<U128>> {
        self.internal_mt_batch_transfer(
            sender_id,
            receiver_id,
            token_ids.clone(),
            amounts.clone(),
            memo,
        );

        let previous_owner_ids = vec![sender_id.clone(); token_ids.len()];

        ext_mt_receiver::ext(receiver_id.clone())
            .mt_on_transfer(
                sender_id.clone(),
                previous_owner_ids.clone(),
                token_ids.clone(),
                amounts.clone(),
                msg,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(MT_RESOLVE_GAS)
                    // do not distribute remaining gas here (so that all that's left goes to `mt_on_transfer`)
                    .with_unused_gas_weight(0)
                    .mt_resolve_transfer(
                        previous_owner_ids,
                        receiver_id.clone(),
                        token_ids,
                        amounts,
                        None,
                    ),
            )
            .into()
    }

    pub(crate) fn get_pool_share_token_id(&self, pool_id: u32) -> String {
        pool_id.to_string()
    }

    pub(crate) fn get_pool_id_from_token_id(&self, token_id: &String) -> Option<u32> {
        token_id.parse::<u32>().ok()
    }
}
