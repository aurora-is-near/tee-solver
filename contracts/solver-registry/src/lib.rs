use dcap_qvl::{verify, QuoteCollateralV3};
use hex::{decode, encode};
use near_sdk::{
    assert_one_yocto,
    env::{self, block_timestamp},
    ext_contract, log, near, require,
    store::{IterableMap, IterableSet, Vector},
    AccountId, NearToken, PanicOnDefault, Promise, PublicKey,
};

use crate::pool::*;
use crate::types::*;

mod admin;
mod collateral;
mod events;
mod pool;
mod token_receiver;
mod types;
mod upgrade;
mod view;

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
        let result = verify::verify(&quote, &collateral, now).expect("report is not verified");
        let rtmr3 = encode(result.report.as_td10().unwrap().rt_mr3.to_vec());
        let codehash = collateral::verify_codehash(tcb_info, rtmr3);

        // only allow workers with approved code hashes to register
        self.assert_approved_codehash(&codehash);

        log!("verify result: {:?}", result);

        // TODO: verify predecessor implicit account is derived from this public key
        let public_key = env::signer_account_pk();

        let predecessor = env::predecessor_account_id();
        self.worker_by_account_id.insert(
            predecessor,
            Worker {
                pool_id,
                checksum,
                codehash,
            },
        );

        // add the public key to the intents vault
        ext_intents_vault::ext(self.get_pool_account_id(pool_id))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .add_public_key(self.intents_contract_id.clone(), public_key)
            .into()
    }

    pub fn require_approved_codehash(&self) {
        let worker = self.get_worker(env::predecessor_account_id());
        self.assert_approved_codehash(&worker.codehash);
    }
}

impl Contract {
    fn assert_approved_codehash(&self, codehash: &String) {
        require!(
            self.approved_codehashes.contains(codehash),
            "Invalid code hash"
        );
    }
}
