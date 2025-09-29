use crate::{Contract, ContractExt, DockerComposeHash, Event, Role, env, require};
use near_plugins::{AccessControllable, access_control_any};
use near_sdk::{Promise, PublicKey, assert_one_yocto, near};

#[near]
impl Contract {
    /// Approve a docker compose hash for worker registration
    ///
    #[access_control_any(roles(Role::Owner))]
    pub fn approve_compose_hash(&mut self, compose_hash: String) {
        DockerComposeHash::try_from_hex(&compose_hash)
            .unwrap_or_else(|_| env::panic_str("Invalid compose hash"));

        self.approved_compose_hashes.insert(compose_hash.clone());

        Event::ComposeHashApproved {
            compose_hash: &compose_hash,
        }
        .emit();
    }

    /// Remove an approved docker compose has
    #[access_control_any(roles(Role::Owner))]
    pub fn remove_compose_hash(&mut self, compose_hash: String) {
        DockerComposeHash::try_from_hex(&compose_hash)
            .unwrap_or_else(|_| env::panic_str("Invalid compose hash"));

        require!(
            self.approved_compose_hashes.remove(&compose_hash),
            "Compose hash not found"
        );

        Event::ComposeHashRemoved {
            compose_hash: &compose_hash,
        }
        .emit();
    }

    #[payable]
    #[access_control_any(roles(Role::Owner))]
    pub fn add_full_access_key(&mut self, public_key: PublicKey) -> Promise {
        assert_one_yocto();
        Promise::new(env::current_account_id()).add_full_access_key(public_key)
    }
}
