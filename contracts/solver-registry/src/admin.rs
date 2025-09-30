use near_plugins::{AccessControllable, access_control_any};
use near_sdk::json_types::U128;
use near_sdk::{
    AccountId, Gas, Promise, PromiseError, PromiseOrValue, PublicKey, assert_one_yocto, near,
};

use crate::ext::ext_intents_vault;
use crate::{Contract, ContractExt, DockerComposeHash, Event, ONE_YOCTO, Role, env, require};

const GAS_WITHDRAW_FROM_POOL_CALLBACK: Gas = Gas::from_tgas(5);

#[near]
impl Contract {
    /// Approve a docker compose hash for worker registration
    #[payable]
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

    /// Remove an approved docker compose hash
    #[payable]
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
        Event::FullAccessKeyAdded {
            public_key: &public_key,
        }
        .emit();
        Promise::new(env::current_account_id()).add_full_access_key(public_key)
    }

    /// Withdraw tokens from a pool to the receiver account id.
    #[payable]
    #[access_control_any(roles(Role::Owner))]
    pub fn withdraw_from_pool(
        &mut self,
        pool_id: u32,
        receiver_id: AccountId,
        token_id: AccountId,
        amount: U128,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        let pool = self
            .pools
            .get(pool_id)
            .unwrap_or_else(|| env::panic_str("Pool not found"));
        require!(pool.token_ids.contains(&token_id), "Invalid token ID");
        // We do not check that the amount does not exceed the pool balance because the solver registry
        // is not tracking the real-time NEAR Intents balance of the pool
        require!(amount.0 > 0, "Invalid amount");

        let pool_account_id = Self::get_pool_account_id(pool_id);
        ext_intents_vault::ext(pool_account_id)
            .with_attached_deposit(ONE_YOCTO)
            .ft_withdraw(
                self.intents_contract_id.clone(),
                token_id.clone(),
                receiver_id,
                amount,
                Some("withdraw from pool".to_string()),
                None,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_WITHDRAW_FROM_POOL_CALLBACK)
                    .with_unused_gas_weight(0)
                    .on_withdraw_from_pool(pool_id, token_id),
            )
            .into()
    }

    #[private]
    pub fn on_withdraw_from_pool(
        &mut self,
        pool_id: u32,
        token_id: AccountId,
        #[callback_result] result: Result<U128, PromiseError>,
    ) -> U128 {
        match result {
            Ok(amount) => {
                Event::AssetWithdrawn {
                    pool_id,
                    token_id: &token_id,
                    amount: &amount,
                }
                .emit();

                amount
            }
            Err(e) => env::panic_str(&format!("Error withdrawing from pool: {e:?}")),
        }
    }
}
