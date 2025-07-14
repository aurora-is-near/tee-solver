use std::borrow::Cow;

use crate::nep245::interface::{
    resolver::MultiTokenResolver, ClearedApproval, MtEventEmit, MtTransferEvent, TokenId,
};
use near_sdk::{env, json_types::U128, near, require, serde_json, AccountId, PromiseResult};

use crate::{Contract, ContractExt};

#[near]
impl MultiTokenResolver for Contract {
    #[private]
    fn mt_resolve_transfer(
        &mut self,
        previous_owner_ids: Vec<AccountId>,
        receiver_id: AccountId,
        token_ids: Vec<TokenId>,
        #[allow(unused_mut)] mut amounts: Vec<U128>,
        approvals: Option<Vec<Option<Vec<ClearedApproval>>>>,
    ) -> Vec<U128> {
        require!(approvals.is_none(), "approvals are not supported");
        require!(
            !token_ids.is_empty()
                && previous_owner_ids.len() == token_ids.len()
                && amounts.len() == token_ids.len(),
            "invalid args"
        );

        let mut refunds = match env::promise_result(0) {
            PromiseResult::Successful(value) => serde_json::from_slice::<Vec<U128>>(&value)
                .ok()
                .filter(|refund| refund.len() == amounts.len())
                .unwrap_or_else(|| amounts.clone()),
            PromiseResult::Failed => amounts.clone(),
        };

        let sender_id = previous_owner_ids
            .first()
            .cloned()
            .unwrap_or_else(|| env::panic_str("Invalid sender"));

        for ((token_id, previous_owner_id), (amount, refund)) in token_ids
            .iter()
            .map(|token_id| {
                token_id
                    .parse()
                    .unwrap_or_else(|_| env::panic_str("Invalid token ID"))
            })
            .zip(previous_owner_ids)
            .zip(amounts.iter_mut().zip(&mut refunds))
        {
            require!(
                sender_id == previous_owner_id,
                "approvals are not supported"
            );

            refund.0 = refund.0.min(amount.0);

            let pool_id = self.get_pool_id_from_token_id(&token_id).unwrap();
            let pool = self.pools.get_mut(pool_id).unwrap();
            let receiver_balance: &u128 = pool.shares.get(&receiver_id).unwrap_or(&0u128);
            // refund maximum what we can
            refund.0 = refund.0.min(*receiver_balance);
            if refund.0 == 0 {
                // noting to refund
                continue;
            }
            // withdraw refund
            pool.shares
                .insert(receiver_id.clone(), *receiver_balance - refund.0);
            // deposit refund
            let sender_balance: &u128 = pool.shares.get(&previous_owner_id).unwrap_or(&0u128);
            pool.shares
                .insert(previous_owner_id.clone(), *sender_balance + refund.0);

            // update as used amount in-place
            amount.0 -= refund.0;
        }

        let (refunded_token_ids, refunded_amounts): (Vec<_>, Vec<_>) = token_ids
            .into_iter()
            .zip(refunds)
            .filter(|(_token_id, refund)| refund.0 > 0)
            .unzip();

        if !refunded_amounts.is_empty() {
            // deposit refunds
            Cow::Borrowed(
                [MtTransferEvent {
                    authorized_id: None,
                    old_owner_id: Cow::Borrowed(&receiver_id),
                    new_owner_id: Cow::Borrowed(&sender_id),
                    token_ids: refunded_token_ids.into(),
                    amounts: refunded_amounts.into(),
                    memo: Some("refund".into()),
                }]
                .as_slice(),
            )
            .emit();
        }

        amounts
    }
}
