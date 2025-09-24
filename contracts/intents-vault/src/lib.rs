use near_sdk::{
    AccountId, NearToken, Promise, PublicKey, assert_one_yocto, env, ext_contract, near, require,
};

#[allow(dead_code)]
#[ext_contract(ext_intents)]
trait IntentsContract {
    fn add_public_key(public_key: PublicKey);
    fn remove_public_key(public_key: PublicKey);
}

#[derive(Default)]
#[near(contract_state)]
pub struct Contract {}

/// TODO: the contract can be deployed as a global contract
#[near]
impl Contract {
    #[payable]
    pub fn add_public_key(
        &mut self,
        intents_contract_id: AccountId,
        public_key: PublicKey,
    ) -> Promise {
        assert_one_yocto();
        require_parent_account();

        ext_intents::ext(intents_contract_id)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .add_public_key(public_key)
    }

    #[payable]
    pub fn remove_public_key(
        &mut self,
        intents_contract_id: AccountId,
        public_key: PublicKey,
    ) -> Promise {
        assert_one_yocto();
        require_parent_account();

        ext_intents::ext(intents_contract_id)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .remove_public_key(public_key)
    }
}

fn require_parent_account() {
    let contract_id = env::current_account_id().to_string();
    let (_, parent_account_id) = contract_id.split_once('.').expect("Invalid contract ID");
    require!(
        env::predecessor_account_id() == parent_account_id,
        "Only parent account can perform this action"
    );
}
