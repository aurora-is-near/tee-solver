use crate::*;
use near_sdk::AccountId;

#[near]
impl Contract {
    pub fn get_worker(&self, account_id: AccountId) -> Worker {
        self.worker_by_account_id
            .get(&account_id)
            .unwrap()
            .to_owned()
    }

    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }
}
