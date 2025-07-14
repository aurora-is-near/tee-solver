use crate::nep245::interface::{enumeration::MultiTokenEnumeration, Token};
use crate::{Contract, ContractExt};
use near_sdk::{json_types::U128, near, AccountId};

#[near]
impl MultiTokenEnumeration for Contract {
    fn mt_tokens(&self, from_index: Option<U128>, limit: Option<u32>) -> Vec<Token> {
        let from_index = from_index.map(|x| x.0 as usize).unwrap_or(0);
        let limit = limit.unwrap_or(50) as usize;

        let mut tokens = Vec::new();
        for pool_id in from_index..self.pools.len() as usize {
            if tokens.len() >= limit {
                break;
            }
            let token_id = self.get_pool_share_token_id(pool_id as u32);
            tokens.push(Token {
                token_id,
                owner_id: None,
            });
        }
        tokens
    }

    fn mt_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u32>,
    ) -> Vec<Token> {
        let from_index = from_index.map(|x| x.0 as usize).unwrap_or(0);
        let limit = limit.unwrap_or(50) as usize;

        let mut tokens = Vec::new();
        let mut count = 0;

        for pool_id in 0..self.pools.len() {
            if count >= from_index {
                if tokens.len() >= limit {
                    break;
                }
                if let Some(pool) = self.pools.get(pool_id) {
                    let shares = pool.shares.get(&account_id).copied().unwrap_or(0);
                    if shares > 0 {
                        let token_id = self.get_pool_share_token_id(pool_id);
                        tokens.push(Token {
                            token_id,
                            owner_id: None,
                        });
                    }
                }
            }
            count += 1;
        }
        tokens
    }
}
