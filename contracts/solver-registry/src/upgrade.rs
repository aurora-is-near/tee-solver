use crate::{Contract, ContractExt};

use near_sdk::{
    AccountId, Gas, GasWeight, NearToken, Promise, PromiseOrValue, assert_one_yocto, env,
    near_bindgen,
};

#[near_bindgen]
impl Contract {
    #[init(ignore_state)]
    #[payable]
    #[private]
    #[allow(clippy::use_self)]
    #[must_use]
    pub fn migrate() -> Self {
        assert_one_yocto();
        env::state_read::<Self>().unwrap_or_else(|| env::panic_str("Failed to read contract state"))
    }

    pub fn upgrade(&mut self) -> PromiseOrValue<AccountId> {
        self.assert_owner();
        let code = env::input().unwrap_or_else(|| env::panic_str("Code not found"));
        Promise::new(env::current_account_id())
            .deploy_contract(code)
            .function_call_weight(
                "migrate".into(),
                vec![],
                NearToken::from_yoctonear(1),
                Gas::from_tgas(0),
                GasWeight(1),
            )
            .function_call_weight(
                "get_owner_id".into(),
                vec![],
                NearToken::ZERO,
                Gas::from_tgas(10),
                GasWeight(0),
            )
            .into()
    }
}
