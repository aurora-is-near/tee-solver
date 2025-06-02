use crate::{Contract, ContractExt};

use near_sdk::{
    assert_one_yocto, env, near_bindgen, AccountId, Gas, GasWeight, NearToken, Promise,
    PromiseOrValue,
};

#[near_bindgen]
impl Contract {
    #[init(ignore_state)]
    #[payable]
    #[private]
    pub fn migrate() -> Self {
        assert_one_yocto();
        env::state_read::<Self>().expect("Failed to read contract state")
    }

    pub fn upgrade(&mut self) -> PromiseOrValue<AccountId> {
        self.assert_owner();
        let code = env::input().expect("Code not found");
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
                NearToken::from_millinear(0),
                Gas::from_tgas(10),
                GasWeight(0),
            )
            .into()
    }
}
