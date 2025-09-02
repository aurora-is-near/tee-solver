extern crate alloc;

use dcap_qvl::{verify, QuoteCollateralV3};
use hex::{decode, encode};
use near_sdk::{
    assert_one_yocto,
    env::{self, block_timestamp, block_timestamp_ms},
    ext_contract, near, require,
    store::{IterableMap, IterableSet, Vector},
    AccountId, Gas, NearToken, PanicOnDefault, Promise, PromiseError, PublicKey,
};

use crate::events::*;
use crate::pool::*;
use crate::types::*;
use crate::attestation::attestation::{Attestation, DstackAttestation};
use crate::attestation::quote::QuoteBytes;
use crate::attestation::collateral::Collateral;
use crate::attestation::report_data::{ReportData, ReportDataV1};
use crate::attestation::hash::{MpcDockerImageHash, LauncherDockerComposeHash};
use dstack_sdk_types::dstack::TcbInfo;
use std::str::FromStr;
use serde_json::Value;

mod admin;
mod attestation;
// mod collateral;
mod events;
mod ext;
mod pool;
mod token_receiver;
pub mod types;
mod upgrade;
mod view;

const GAS_REGISTER_WORKER_CALLBACK: Gas = Gas::from_tgas(10);

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Worker {
    pool_id: u32,
    checksum: String,
    codehash: String,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    intents_contract_id: AccountId,
    pools: Vector<Pool>,
    approved_codehashes: IterableSet<String>,
    worker_by_account_id: IterableMap<AccountId, Worker>,
    worker_ping_timeout_ms: TimestampMs,
}

#[allow(dead_code)]
#[ext_contract(ext_intents_vault)]
trait IntentsVaultContract {
    fn add_public_key(intents_contract_id: AccountId, public_key: PublicKey);
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn new(
        owner_id: AccountId,
        intents_contract_id: AccountId,
        worker_ping_timeout_ms: TimestampMs,
    ) -> Self {
        Self {
            owner_id,
            intents_contract_id,
            pools: Vector::new(Prefix::Pools),
            approved_codehashes: IterableSet::new(Prefix::ApprovedCodeHashes),
            worker_by_account_id: IterableMap::new(Prefix::WorkerByAccountId),
            worker_ping_timeout_ms,
        }
    }

    #[payable]
    pub fn register_worker(
        &mut self,
        pool_id: u32,
        quote_hex: String,
        collateral: String,
        checksum: String,
        tcb_info: String,
    ) -> Promise {
        assert_one_yocto();
        let pool = self.pools.get(pool_id).expect("Pool not found");
        let worker_id = env::predecessor_account_id();
        // register new worker is allowed only if there's no active worker and the worker is not already registered
        require!(
            !pool.has_active_worker(self.worker_ping_timeout_ms),
            "Only one active worker is allowed per pool"
        );
        require!(
            pool.worker_id.is_none() || pool.worker_id.as_ref().unwrap() != &worker_id,
            "Worker already registered"
        );

        // Get the signer's public key
        let public_key = env::signer_account_pk();

        // Parse the attestation components
        let quote_bytes = QuoteBytes::from(decode(&quote_hex).expect("Invalid quote hex"));
        let collateral_data = Collateral::from_str(&collateral).expect("Invalid collateral format");
        let tcb_info_data: TcbInfo = serde_json::from_str(&tcb_info)
            .expect("Invalid TCB info format");

        // Create the attestation
        let attestation = Attestation::Dstack(DstackAttestation::new(
            quote_bytes,
            collateral_data,
            tcb_info_data,
        ));

        // Create expected report data from the public key
        let expected_report_data = ReportData::V1(ReportDataV1::new(public_key.clone()));

        // Get current timestamp in seconds
        let timestamp_s = block_timestamp() / 1_000_000_000;

        // For now, allow all hashes (you can configure this based on your security requirements)
        let allowed_mpc_docker_image_hashes: Vec<MpcDockerImageHash> = vec![];
        let allowed_launcher_docker_compose_hashes: Vec<LauncherDockerComposeHash> = vec![];

        // Verify the attestation
        require!(
            attestation.verify(
                expected_report_data,
                timestamp_s,
                &allowed_mpc_docker_image_hashes,
                &allowed_launcher_docker_compose_hashes,
            ),
            "Attestation verification failed"
        );

        // Extract codehash from TCB info for approved code hash verification
        let codehash = self.extract_codehash_from_tcb_info(&tcb_info);
        self.assert_approved_codehash(&codehash);

        // add the public key to the intents vault
        ext_intents_vault::ext(self.get_pool_account_id(pool_id))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .add_public_key(self.intents_contract_id.clone(), public_key.clone())
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_REGISTER_WORKER_CALLBACK)
                    .on_worker_key_added(worker_id, pool_id, public_key, codehash, checksum),
            )
    }

    #[private]
    pub fn on_worker_key_added(
        &mut self,
        worker_id: AccountId,
        pool_id: u32,
        public_key: PublicKey,
        codehash: String,
        checksum: String,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_ok() {
            self.worker_by_account_id.insert(
                worker_id.clone(),
                Worker {
                    pool_id,
                    checksum: checksum.clone(),
                    codehash: codehash.clone(),
                },
            );

            // Update the pool with the worker ID and last ping timestamp
            let pool = self.pools.get_mut(pool_id).expect("Pool not found");
            pool.worker_id = Some(worker_id.clone());
            pool.last_ping_timestamp_ms = block_timestamp_ms();
            self.pools.flush();

            Event::WorkerRegistered {
                worker_id: &worker_id,
                pool_id: &pool_id,
                public_key: &public_key,
                codehash: &codehash,
                checksum: &checksum,
            }
            .emit();
        }
    }

    /// Heartbeat to notify the pool that the worker is still alive.
    pub fn ping(&mut self) {
        let worker_id = env::predecessor_account_id();
        let worker = self
            .get_worker(worker_id.clone())
            .expect("Worker not found");
        self.assert_approved_codehash(&worker.codehash);
        let pool = self.pools.get_mut(worker.pool_id).expect("Pool not found");
        let registered_worker_id = pool.worker_id.as_ref().expect("Worker not registered");
        require!(
            registered_worker_id == &worker_id,
            "Only the registered worker can ping"
        );

        pool.last_ping_timestamp_ms = block_timestamp_ms();
        self.pools.flush();

        Event::WorkerPinged {
            pool_id: &worker.pool_id,
            worker_id: &worker_id,
            timestamp_ms: &block_timestamp_ms(),
        }
        .emit();
    }
}

impl Contract {
    fn assert_approved_codehash(&self, codehash: &String) {
        require!(
            self.approved_codehashes.contains(codehash),
            "Invalid code hash"
        );
    }

    /// Extract codehash from TCB info for verification
    fn extract_codehash_from_tcb_info(&self, tcb_info: &str) -> String {
        // Parse the TCB info to extract the codehash
        // This is a simplified implementation - you may need to adjust based on your TCB info structure
        let tcb_info_data: Value = serde_json::from_str(tcb_info)
            .expect("Invalid TCB info JSON format");
        
        // Try to extract codehash from various possible locations in the TCB info
        // The actual structure may vary depending on your TEE implementation
        if let Some(codehash) = tcb_info_data["codehash"].as_str() {
            return codehash.to_string();
        }
        
        if let Some(codehash) = tcb_info_data["app_compose"]["codehash"].as_str() {
            return codehash.to_string();
        }
        
        if let Some(codehash) = tcb_info_data["measurements"]["codehash"].as_str() {
            return codehash.to_string();
        }
        
        // If no codehash is found, use the checksum as a fallback
        // This is a temporary solution - you should implement proper codehash extraction
        "default_codehash".to_string()
    }
}
