use near_plugins::{Pausable, pause};
use near_sdk::json_types::U128;
use near_sdk::{AccountId, PromiseOrValue, near};

use crate::{Contract, ContractExt, env};

const ERR_MALFORMED_MESSAGE: &str = "Invalid transfer action message";

#[near(serializers=[json])]
enum TokenReceiverMessage {
    DepositIntoPool { pool_id: u32 },
}

#[near]
impl Contract {
    #[pause]
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        if msg.is_empty() {
            // refund all
            return PromiseOrValue::Value(amount);
        }

        let token_id = env::predecessor_account_id();
        let message = near_sdk::serde_json::from_str::<TokenReceiverMessage>(&msg)
            .unwrap_or_else(|_| env::panic_str(ERR_MALFORMED_MESSAGE));
        match message {
            TokenReceiverMessage::DepositIntoPool { pool_id } => {
                self.deposit_into_pool(pool_id, &token_id, &sender_id, amount.0)
            }
        }
    }
}
