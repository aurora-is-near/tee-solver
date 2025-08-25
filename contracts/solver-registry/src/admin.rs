use crate::*;
use near_sdk::near;

#[near]
impl Contract {
    pub fn approve_codehash(&mut self, codehash: String) {
        self.assert_owner();
        self.approved_codehashes.insert(codehash.clone());

        Event::CodehashApproved {
            codehash: &codehash,
        }
        .emit();
    }

    pub fn remove_codehash(&mut self, codehash: String) {
        self.assert_owner();
        self.approved_codehashes.remove(&codehash);

        Event::CodehashRemoved {
            codehash: &codehash,
        }
        .emit();
    }

    pub fn change_owner(&mut self, new_owner_id: AccountId) {
        self.assert_owner();
        let old_owner_id = self.owner_id.clone();
        self.owner_id = new_owner_id.clone();

        Event::OwnerChanged {
            old_owner_id: &old_owner_id,
            new_owner_id: &new_owner_id,
        }
        .emit();
    }
}

impl Contract {
    pub(crate) fn assert_owner(&mut self) {
        require!(env::predecessor_account_id() == self.owner_id);
    }
}
