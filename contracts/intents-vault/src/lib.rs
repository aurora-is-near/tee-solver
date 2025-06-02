use near_sdk::{assert_one_yocto, env, near, require, AccountId, Promise, PublicKey};

#[ext_contract(ext_intents)]
trait IntentsContract {
    fn add_public_key(public_key: PublicKey);
}

#[near]
impl Contract {
    #[payable]
    pub fn add_public_key(
        &mut self,
        intents_contract_id: AccountId,
        public_key: PublicKey,
    ) -> Promise {
        assert_one_yocto();
        self.require_parent_account();

        ext_intents::ext(intents_contract_id).add_public_key(public_key)
    }
}

impl Contract {
    fn require_parent_account(&mut self) {
        let parent_account_id = env::current_account_id().split('.').next().unwrap();
        require!(env::predecessor_account_id() == parent_account_id);
    }
}
