use near_sdk::store::LookupMap;
use near_sdk::{
    assert_one_yocto, env, near, AccountId, BorshStorageKey, PanicOnDefault, PublicKey,
};
use std::collections::HashSet;

mod token_receiver;

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    public_keys: LookupMap<AccountId, HashSet<PublicKey>>,
}

#[near]
#[derive(BorshStorageKey)]
pub enum Prefix {
    PublicKeys,
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn new() -> Self {
        Self {
            public_keys: LookupMap::new(Prefix::PublicKeys),
        }
    }

    #[payable]
    pub fn add_public_key(&mut self, public_key: PublicKey) {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();
        let mut keys = self.internal_get_account(&account_id);
        keys.insert(public_key);
        self.public_keys.insert(account_id, keys.clone());
    }

    #[payable]
    pub fn remove_public_key(&mut self, public_key: PublicKey) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let mut keys = self.internal_get_account(&account_id);
        keys.remove(&public_key);
        self.public_keys.insert(account_id, keys.clone());
    }

    pub fn public_keys_of(&self, account_id: AccountId) -> HashSet<PublicKey> {
        self.internal_get_account(&account_id)
    }
}

impl Contract {
    fn internal_get_account(&self, account_id: &AccountId) -> HashSet<PublicKey> {
        self.public_keys
            .get(account_id)
            .unwrap_or(&HashSet::new())
            .clone()
    }
}
