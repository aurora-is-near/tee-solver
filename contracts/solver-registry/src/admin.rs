use crate::*;
use near_sdk::near;

#[near]
impl Contract {
    pub fn approve_codehash(&mut self, codehash: String) {
        self.assert_owner();
        self.approved_codehashes.insert(codehash);
    }

    pub fn change_owner(&mut self, new_owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = new_owner_id;
    }
}

impl Contract {
    pub(crate) fn assert_owner(&mut self) {
        require!(env::predecessor_account_id() == self.owner_id);
    }
}
