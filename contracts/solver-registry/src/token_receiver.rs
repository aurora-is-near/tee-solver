use near_sdk::json_types::U128;
use near_sdk::{near, AccountId, PromiseOrValue};

use crate::*;

const ERR_MALFORMED_MESSAGE: &str = "Invalid transfer action message";

#[near(serializers=[json])]
enum TokenReceiverMessage {
    DepositIntoPool { pool_id: u32 },
}

#[near]
impl Contract {
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
        let message =
            serde_json::from_str::<TokenReceiverMessage>(&msg).expect(ERR_MALFORMED_MESSAGE);
        match message {
            TokenReceiverMessage::DepositIntoPool { pool_id } => {
                self.deposit_into_pool(pool_id, &token_id, &sender_id, amount.0)
            }
        }
    }
}
