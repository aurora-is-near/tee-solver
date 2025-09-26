extern crate alloc;

use dstack_sdk_types::dstack::TcbInfo;
use hex::decode;
use near_plugins::{
    AccessControlRole, AccessControllable, Pausable, Upgradable, access_control, pause,
};
use near_sdk::{
    AccountId, Gas, NearToken, PanicOnDefault, Promise, PromiseError, PublicKey, assert_one_yocto,
    borsh::BorshDeserialize,
    env::{self, block_timestamp_ms, sha256},
    near, require,
    store::{IterableMap, IterableSet, Vector},
};
use std::str::FromStr;

use crate::attestation::{
    app_compose::AppCompose,
    attestation::{Attestation, DstackAttestation},
    collateral::Collateral,
    hash::{DockerComposeHash, DockerImageHash},
    quote::QuoteBytes,
    report_data::ReportData,
};
use crate::events::Event;
use crate::ext::ext_intents_vault;
use crate::pool::{Pool, PoolInfo};
use crate::types::{Balance, Prefix, TimestampMs};

mod admin;
mod attestation;
mod events;
mod ext;
pub mod pool;
mod token_receiver;
pub mod types;
mod view;

const GAS_ADD_WORKER_KEY: Gas = Gas::from_tgas(20);
const GAS_REMOVE_WORKER_KEY: Gas = Gas::from_tgas(20);
const GAS_ADD_WORKER_KEY_CALLBACK: Gas = Gas::from_tgas(10);
const GAS_REMOVE_WORKER_KEY_CALLBACK: Gas = Gas::from_tgas(20) // 20 Tgas for the callback function itself
    .saturating_add(GAS_ADD_WORKER_KEY)
    .saturating_add(GAS_ADD_WORKER_KEY_CALLBACK);

#[derive(AccessControlRole, Clone, Copy)]
#[near(serializers = [json])]
enum Role {
    Owner,
    Dao,
    PauseManager,
    UnpauseManager,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Worker {
    pub pool_id: u32,
    pub checksum: String,
    pub compose_hash: String,
    pub public_key: PublicKey,
}

#[derive(PanicOnDefault, Pausable, Upgradable)]
#[access_control(role_type(Role))]
#[upgradable(access_control_roles(
    code_stagers(Role::Owner),
    code_deployers(Role::Owner),
    duration_initializers(Role::Owner),
    duration_update_stagers(Role::Owner),
    duration_update_appliers(Role::Owner),
))]
#[pausable(
    pause_roles(Role::Owner, Role::PauseManager),
    unpause_roles(Role::Owner, Role::UnpauseManager)
)]
#[near(contract_state)]
pub struct Contract {
    intents_contract_id: AccountId,
    pools: Vector<Pool>,
    approved_compose_hashes: IterableSet<String>,
    worker_by_account_id: IterableMap<AccountId, Worker>,
    worker_ping_timeout_ms: TimestampMs,
}

#[near]
impl Contract {
    #[init]
    #[private]
    #[must_use]
    #[allow(clippy::use_self)]
    pub fn new(
        owner_id: AccountId,
        intents_contract_id: AccountId,
        worker_ping_timeout_ms: TimestampMs,
    ) -> Self {
        let mut contract = Self {
            intents_contract_id,
            pools: Vector::new(Prefix::Pools),
            approved_compose_hashes: IterableSet::new(Prefix::ApprovedComposeHashes),
            worker_by_account_id: IterableMap::new(Prefix::WorkerByAccountId),
            worker_ping_timeout_ms,
        };

        let mut acl = contract.acl_get_or_init();

        acl.add_super_admin_unchecked(&owner_id);
        acl.grant_role_unchecked(Role::Owner, &owner_id);

        contract
    }

    /// Register worker with TEE attestation. The worker needs to running inside a CVM with one of the approved docker compose hashes.
    ///
    /// The current TEE attestation module reuses the implementation from [NEAR MPC](https://github.com/near/mpc) TEE attestation with slight change.
    /// Find more details about TEE attestation module in `attestation/README.md`.
    #[payable]
    #[pause]
    pub fn register_worker(
        &mut self,
        pool_id: u32,
        quote_hex: String,
        collateral: String,
        checksum: String,
        tcb_info: String,
    ) -> Promise {
        assert_one_yocto();
        let pool = self
            .pools
            .get(pool_id)
            .unwrap_or_else(|| env::panic_str("Pool not found"));

        // Register new worker is allowed only if there's no active worker and the worker is not already registered
        let worker_id = env::predecessor_account_id();
        require!(
            !pool.has_active_worker(self.worker_ping_timeout_ms),
            "Only one active worker is allowed per pool"
        );
        require!(
            pool.worker_id.is_none() || pool.worker_id.as_deref() != Some(&worker_id),
            "Worker already registered"
        );

        // Parse the attestation components
        let quote_bytes = QuoteBytes::from(
            decode(&quote_hex).unwrap_or_else(|_| env::panic_str("Invalid quote hex")),
        );
        let collateral_data = Collateral::from_str(&collateral)
            .unwrap_or_else(|_| env::panic_str("Invalid collateral format"));
        let tcb_info_data: TcbInfo = near_sdk::serde_json::from_str(&tcb_info)
            .unwrap_or_else(|_| env::panic_str("Invalid TCB info format"));

        // Create the attestation
        let attestation = Attestation::Dstack(DstackAttestation::new(
            quote_bytes,
            collateral_data,
            tcb_info_data.clone(),
        ));

        // Get the signer's public key
        let public_key = env::signer_account_pk();
        // Create expected report data from the public key
        let expected_report_data = ReportData::new(public_key.clone());

        // Get current timestamp in seconds
        let timestamp_s = block_timestamp_ms() / 1_000;

        // For now, allow all docker image hashes as we only verify the docker compose hash
        let allowed_docker_image_hashes: Vec<DockerImageHash> = vec![];
        let allowed_docker_compose_hashes: Vec<DockerComposeHash> = self
            .approved_compose_hashes
            .iter()
            .map(|hash| {
                DockerComposeHash::try_from_hex(hash)
                    .unwrap_or_else(|_| env::panic_str("Invalid compose hash"))
            })
            .collect();

        // Verify the attestation
        require!(
            attestation.verify(
                expected_report_data,
                timestamp_s,
                &allowed_docker_image_hashes,
                &allowed_docker_compose_hashes,
            ),
            "Attestation verification failed"
        );

        // Extract docker compose hash from TCB info
        let docker_compose_hash =
            Self::find_approved_compose_hash(&tcb_info_data, &allowed_docker_compose_hashes)
                .unwrap_or_else(|| env::panic_str("Invalid docker compose hash"));
        let docker_compose_hash_hex = docker_compose_hash.as_hex();

        // Remove the public key of the inactive worker if exists
        if let Some(inactive_worker_id) = pool.worker_id.as_ref() {
            let inactive_worker = self
                .worker_by_account_id
                .get(inactive_worker_id)
                .unwrap_or_else(|| env::panic_str("Worker not registered"));
            ext_intents_vault::ext(Self::get_pool_account_id(pool_id))
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .with_static_gas(GAS_REMOVE_WORKER_KEY)
                .with_unused_gas_weight(0)
                .remove_public_key(
                    self.intents_contract_id.clone(),
                    inactive_worker.public_key.clone(),
                )
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(GAS_REMOVE_WORKER_KEY_CALLBACK)
                        .with_unused_gas_weight(0)
                        .on_inactive_worker_key_removed(
                            worker_id,
                            pool_id,
                            &public_key,
                            docker_compose_hash_hex,
                            checksum,
                        ),
                )
        } else {
            self.register_new_public_key(
                worker_id,
                pool_id,
                &public_key,
                docker_compose_hash_hex,
                checksum,
            )
        }
    }

    #[private]
    pub fn on_inactive_worker_key_removed(
        &mut self,
        worker_id: AccountId,
        pool_id: u32,
        public_key: &PublicKey,
        docker_compose_hash_hex: String,
        checksum: String,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Promise {
        if call_result.is_ok() {
            // remove inactive worker
            let pool = self
                .pools
                .get(pool_id)
                .unwrap_or_else(|| env::panic_str("Pool not found"));
            let inactive_worker_id = pool
                .worker_id
                .as_ref()
                .unwrap_or_else(|| env::panic_str("Pool has no worker"));
            let inactive_worker = self
                .worker_by_account_id
                .remove(inactive_worker_id)
                .unwrap_or_else(|| env::panic_str("Worker not registered"));
            Event::WorkerRemoved {
                worker_id: inactive_worker_id,
                pool_id: &pool_id,
                public_key: &inactive_worker.public_key,
                compose_hash: &inactive_worker.compose_hash,
                checksum: &inactive_worker.checksum,
            }
            .emit();

            // register new worker and its key
            self.register_new_public_key(
                worker_id,
                pool_id,
                public_key,
                docker_compose_hash_hex,
                checksum,
            )
        } else {
            env::panic_str("Failed to remove inactive worker key");
        }
    }

    #[private]
    pub fn on_worker_key_added(
        &mut self,
        worker_id: AccountId,
        pool_id: u32,
        public_key: &PublicKey,
        compose_hash: String,
        checksum: String,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_ok() {
            self.worker_by_account_id.insert(
                worker_id.clone(),
                Worker {
                    pool_id,
                    checksum: checksum.clone(),
                    compose_hash: compose_hash.clone(),
                    public_key: public_key.clone(),
                },
            );

            // Update the pool with the worker ID and last ping timestamp
            let pool = self
                .pools
                .get_mut(pool_id)
                .unwrap_or_else(|| env::panic_str("Pool not found"));
            pool.worker_id = Some(worker_id.clone());
            pool.last_ping_timestamp_ms = block_timestamp_ms();
            self.pools.flush();

            Event::WorkerRegistered {
                worker_id: &worker_id,
                pool_id: &pool_id,
                public_key,
                compose_hash: &compose_hash,
                checksum: &checksum,
            }
            .emit();
        }
    }

    /// Heartbeat to notify the pool that the worker is still alive.
    pub fn ping(&mut self) {
        let worker_id = env::predecessor_account_id();
        let worker = self
            .get_worker(&worker_id)
            .unwrap_or_else(|| env::panic_str("Worker not found"));
        self.assert_approved_compose_hash(&worker.compose_hash);

        let pool = self
            .pools
            .get_mut(worker.pool_id)
            .unwrap_or_else(|| env::panic_str("Pool not found"));
        let registered_worker_id = pool
            .worker_id
            .as_ref()
            .unwrap_or_else(|| env::panic_str("Worker not registered"));
        require!(
            registered_worker_id == &worker_id,
            "Only the registered worker can ping"
        );

        let block_timestamp_ms = block_timestamp_ms();

        pool.last_ping_timestamp_ms = block_timestamp_ms;
        self.pools.flush();

        Event::WorkerPinged {
            pool_id: &worker.pool_id,
            worker_id: &worker_id,
            timestamp_ms: &block_timestamp_ms,
        }
        .emit();
    }
}

impl Contract {
    fn assert_approved_compose_hash(&self, compose_hash: &String) {
        require!(
            self.approved_compose_hashes.contains(compose_hash),
            "Invalid compose hash"
        );
    }

    fn find_approved_compose_hash(
        tcb_info: &TcbInfo,
        allowed_hashes: &[DockerComposeHash],
    ) -> Option<DockerComposeHash> {
        let app_compose: AppCompose = match near_sdk::serde_json::from_str(&tcb_info.app_compose) {
            Ok(compose) => compose,
            Err(e) => {
                tracing::error!("Failed to parse app_compose JSON: {:?}", e);
                return None;
            }
        };
        let compose_hash = sha256(app_compose.docker_compose_file.as_bytes());
        allowed_hashes
            .iter()
            .find(|hash| hash.as_hex() == hex::encode(&compose_hash))
            .cloned()
    }

    fn register_new_public_key(
        &self,
        worker_id: AccountId,
        pool_id: u32,
        public_key: &PublicKey,
        docker_compose_hash_hex: String,
        checksum: String,
    ) -> Promise {
        // Add the public key to the intents vault
        ext_intents_vault::ext(Self::get_pool_account_id(pool_id))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(GAS_ADD_WORKER_KEY)
            .with_unused_gas_weight(0)
            .add_public_key(self.intents_contract_id.clone(), public_key.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_ADD_WORKER_KEY_CALLBACK)
                    .with_unused_gas_weight(0)
                    .on_worker_key_added(
                        worker_id,
                        pool_id,
                        public_key,
                        docker_compose_hash_hex,
                        checksum,
                    ),
            )
    }
}
