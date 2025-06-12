use dcap_qvl::{verify, QuoteCollateralV3};
use hex::{decode, encode};
use near_sdk::{
    assert_one_yocto,
    env::{self, block_timestamp},
    ext_contract, log, near, require,
    store::{IterableMap, IterableSet, Vector},
    AccountId, Gas, NearToken, PanicOnDefault, Promise, PromiseError, PublicKey,
};

use crate::events::*;
use crate::pool::*;
use crate::types::*;

mod admin;
mod collateral;
mod events;
mod ext;
mod pool;
mod token_receiver;
mod types;
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
    pub fn new(owner_id: AccountId, intents_contract_id: AccountId) -> Self {
        Self {
            owner_id,
            intents_contract_id,
            pools: Vector::new(Prefix::Pools),
            approved_codehashes: IterableSet::new(Prefix::ApprovedCodeHashes),
            worker_by_account_id: IterableMap::new(Prefix::WorkerByAccountId),
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
        require!(self.has_pool(pool_id), "Pool not found");

        let collateral = collateral::get_collateral(collateral);
        let quote = decode(quote_hex).unwrap();
        let now = block_timestamp() / 1000000000;
        let result = verify::verify(&quote, &collateral, now).expect("Report is not verified");
        let report = result.report.as_td10().unwrap();
        let rtmr3 = encode(report.rt_mr3);

        // verify the signer public key is the same as the one included in the report data
        let report_data = encode(report.report_data);
        let public_key = env::signer_account_pk();
        let public_key_str: String = (&public_key).into();
        // pad the public key hex with 0 to 128 characters
        let public_key_hex = format!("{:0>128}", encode(public_key_str));
        require!(
            public_key_hex == report_data,
            format!(
                "Invalid public key: {} v.s. {}",
                public_key_hex, report_data
            )
        );

        // only allow workers with approved code hashes to register
        let codehash = collateral::verify_codehash(tcb_info, rtmr3);
        self.assert_approved_codehash(&codehash);

        log!("verify result: {:?}", result);

        // TODO: verify predecessor implicit account is derived from this public key

        let worker_id = env::predecessor_account_id();

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
}

impl Contract {
    fn assert_approved_codehash(&self, codehash: &String) {
        require!(
            self.approved_codehashes.contains(codehash),
            "Invalid code hash"
        );
    }

    // fn require_approved_codehash(&self) {
    //     let worker = self.get_worker(env::predecessor_account_id());
    //     self.assert_approved_codehash(&worker.codehash);
    // }
}
