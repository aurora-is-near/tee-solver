use near_sdk::json_types::U128;
use near_sdk::{require, AccountId, Balance, Gas, LookupMap, NearToken, U128};

type Balance = u128;
const CREATE_POOL_STORAGE_DEPOSIT: NearToken = NearToken::parse_near_amount("1");

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Pool {
    /// List of tokens in the pool.
    pub token_ids: Vec<AccountId>,
    /// How much NEAR this contract has.
    pub amounts: Vec<Balance>,
    /// Fee charged for swap in basis points
    pub fee: u64,
    /// Shares of the pool by liquidity providers.
    pub shares: LookupMap<AccountId, Balance>,
    /// Total number of shares.
    pub shares_total_supply: Balance,
}

impl Pool {
    pub fn new(token_ids: Vec<AccountId>, fee: u64) -> Self {
        require!(token_ids.len() == 2, "Must have exactly 2 tokens");
        require!(fee < 10_000, "Fee must be less than 100%");

        Self {
            token_ids,
            amounts: vec![0; token_ids.len()],
            fee,
            shares: LookupMap::new(Prefix::Shares),
            shares_total_supply: 0,
        }
    }
}

#[near]
impl Contract {
    #[payable]
    pub fn create_liquidity_pool(&mut self, tokens: Vec<AccountId>, fee: u64) -> u64 {
        require!(tokens.len() == 2, "Must have exactly 2 tokens");
        require!(fee <= 10000, "Fee must be less than or equal to 100%");

        let pool_id = self.pools.len();
        let pool = Pool::new(token_ids, fee);
        self.pools.push(pool);
        self.pools.flush();

        // TODO: create sub account for managing intents assets

        let pool_account_id = self.get_pool_account_id(pool_id);
        Promise::new(pool_account_id.clone())
            .create_account()
            .transfer()
            .deploy_contract(b"pool.wasm")
            .function_call("new".to_string(), borsh::to_vec(&pool).unwrap())
            .transact();

        pool_id
    }

    #[payable]
    pub fn add_liquidity(
        &mut self,
        pool_id: u64,
        token_ids: Vec<AccountId>,
        amounts: Vec<Balance>,
    ) {
        require!(token_ids.len() == 2, "Must have exactly 2 tokens");
        require!(amounts.len() == 2, "Must have exactly 2 amounts");
        require!(amounts[0] > 0, "Amount must be greater than 0");
        require!(amounts[1] > 0, "Amount must be greater than 0");

        let pool = self.pools.get(pool_id).expect("Pool not found");
        let shares_total_supply = pool.shares_total_supply;
    }

    #[payable]
    pub fn remove_liquidity(&mut self, pool_id: u64, shares: U128) {
        require!(shares > 0, "Shares must be greater than 0");

        let pool = self.pools.get(pool_id).expect("Pool not found");
        let shares_total_supply = pool.shares_total_supply;
        let shares_to_remove = shares.0;
        let amount_to_remove = shares_to_remove * shares_total_supply / pool.shares_total_supply;

        pool.shares_total_supply -= shares_to_remove;
    }
}

impl Contract {
    fn get_pool_account_id(&self, pool_id: u64) -> AccountId {
        format!("pool-{}.{}", pool_id, env::current_account_id())
    }

    fn has_pool(&self, pool_id: u64) -> bool {
        self.pools.get(pool_id).is_some()
    }
}
