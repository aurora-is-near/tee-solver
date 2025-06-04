use crate::*;
use near_sdk::near;

#[near]
impl Contract {
    pub fn approve_codehash(&mut self, codehash: String) {
        self.assert_owner();
        self.approved_codehashes.insert(codehash);
    }
}

impl Contract {
    pub(crate) fn assert_owner(&mut self) {
        require!(env::predecessor_account_id() == self.owner_id);
    }
}
