use near_sdk::json_types::U128;
use near_sdk::{near, AccountId};

use crate::*;

#[near]
impl Contract {
    #[payable]
    pub fn ft_on_transfer(&mut self, token_id: AccountId, amount: U128) {
        // TODO
    }
}
