use crate::*;
use near_sdk::near;

#[near]
impl Contract {
    /// Approve a docker compose hash for worker registration
    pub fn approve_compose_hash(&mut self, compose_hash: String) {
        self.assert_owner();

        self.approved_compose_hashes.insert(compose_hash.clone());

        Event::ComposeHashApproved {
            compose_hash: &compose_hash,
        }
        .emit();
    }

    /// Remove an approved docker compose hash
    pub fn remove_compose_hash(&mut self, compose_hash: String) {
        self.assert_owner();

        self.approved_compose_hashes.remove(&compose_hash);

        Event::ComposeHashRemoved {
            compose_hash: &compose_hash,
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
