use near_sdk::{json_types::U128, log, serde::Serialize, serde_json::json, AccountId, PublicKey};

use crate::types::TimestampMs;

pub const EVENT_STANDARD: &str = "solver-registry";
pub const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize)]
#[serde(
    crate = "near_sdk::serde",
    rename_all = "snake_case",
    tag = "event",
    content = "data"
)]
#[must_use = "Don't forget to `.emit()` this event"]
pub enum Event<'a> {
    WorkerRegistered {
        worker_id: &'a AccountId,
        pool_id: &'a u32,
        public_key: &'a PublicKey,
        compose_hash: &'a String,
        checksum: &'a String,
    },
    WorkerRemoved {
        worker_id: &'a AccountId,
        pool_id: &'a u32,
        public_key: &'a PublicKey,
        compose_hash: &'a String,
        checksum: &'a String,
    },
    CreateLiquidityPool {
        pool_id: &'a u32,
        token_ids: &'a Vec<AccountId>,
        fee: &'a u32,
    },
    WorkerPinged {
        pool_id: &'a u32,
        worker_id: &'a AccountId,
        timestamp_ms: &'a TimestampMs,
    },
    ComposeHashApproved {
        compose_hash: &'a String,
    },
    ComposeHashRemoved {
        compose_hash: &'a String,
    },
    OwnerChanged {
        old_owner_id: &'a AccountId,
        new_owner_id: &'a AccountId,
    },
    AssetDeposited {
        pool_id: &'a u32,
        token_id: &'a AccountId,
        amount: &'a U128,
    },
    AssetWithdrawn {
        pool_id: &'a u32,
        token_id: &'a AccountId,
        amount: &'a U128,
    },
}

impl Event<'_> {
    pub fn emit(&self) {
        let json = json!(self);
        let event_json = json!({
            "standard": EVENT_STANDARD,
            "version": EVENT_STANDARD_VERSION,
            "event": json["event"],
            "data": [json["data"]]
        })
        .to_string();
        log!("EVENT_JSON:{}", event_json);
    }
}
