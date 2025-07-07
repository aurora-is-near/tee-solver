use near_sdk::store::IterableMap;
use near_sdk::{json_types::U128, near, AccountId};

use crate::*;

const ERR_INSUFFICIENT_BALANCE: &str = "Insufficient balance";

#[near(serializers = [borsh])]
pub struct Account {
    /// Deposited NEP-141 token balances: token_id -> balance
    pub deposits: IterableMap<AccountId, Balance>,
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Self {
            deposits: IterableMap::new(Prefix::AccountDeposits {
                account_id: account_id.clone(),
            }),
        }
    }

    pub fn deposit(&mut self, token_id: AccountId, amount: Balance) {
        let balance = self.deposits.get(&token_id).unwrap_or(&0u128);
        self.deposits.insert(token_id, balance + amount);
    }

    pub fn withdraw(&mut self, token_id: AccountId, amount: Balance) {
        let balance = self.deposits.get(&token_id).unwrap_or(&0u128);
        require!(balance >= &amount, ERR_INSUFFICIENT_BALANCE);
        self.deposits.insert(token_id, balance - amount);
    }
}

#[near]
impl Contract {
    /// Deposits certain amount of token into an account
    pub(crate) fn deposit_into_account(
        &mut self,
        account_id: AccountId,
        token_id: AccountId,
        amount: Balance,
    ) {
        let mut account = self
            .accounts
            .remove(&account_id)
            .unwrap_or_else(|| Account::new(&account_id));
        account.deposit(token_id.clone(), amount);
        self.accounts.insert(account_id.clone(), account);
        Event::Deposit {
            account_id: &account_id,
            token_id: &token_id,
            amount: &U128(amount),
        }
        .emit();
    }

    pub(crate) fn withdraw_from_account(
        &mut self,
        account_id: AccountId,
        token_id: AccountId,
        amount: Balance,
    ) {
        let mut account = self
            .accounts
            .remove(&account_id)
            .unwrap_or_else(|| Account::new(&account_id));
        account.withdraw(token_id.clone(), amount);
        self.accounts.insert(account_id.clone(), account);
    }
}
