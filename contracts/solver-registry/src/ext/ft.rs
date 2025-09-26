use near_sdk::{AccountId, PromiseOrValue, ext_contract, json_types::U128};

#[allow(dead_code)]
#[ext_contract(ext_ft)]
trait FungibleTokenContract {
    fn ft_transfer_call(
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;
}
