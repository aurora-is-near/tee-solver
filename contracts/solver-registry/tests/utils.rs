use near_contract_standards::fungible_token::{metadata::FungibleTokenMetadata, Balance};
use near_gas::NearGas;
use near_sdk::{json_types::U128, near, AccountId, NearToken};
use near_workspaces::{network::Sandbox, result::ExecutionFinalResult, Account, Contract, Worker};
use serde_json::json;

pub const SOLVER_REGISTRY_CONTRACT_WASM: &str =
    "../../target/near/solver_registry/solver_registry.wasm";
pub const MOCK_INTENTS_CONTRACT_WASM: &str = "../../target/near/mock_intents/mock_intents.wasm";
pub const MOCK_FT_CONTRACT_WASM: &str = "../../target/near/mock_ft/mock_ft.wasm";

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct WorkerInfo {
    pub pool_id: u32,
    pub checksum: String,
    pub codehash: String,
}

#[near(serializers = [json])]
#[derive(Clone)]
pub struct PoolInfo {
    pub token_ids: Vec<AccountId>,
    pub amounts: Vec<U128>,
    pub fee: u32,
    pub shares_total_supply: U128,
}

pub async fn create_account(
    sandbox: &Worker<Sandbox>,
    prefix: &str,
    balance: Balance,
) -> Result<Account, Box<dyn std::error::Error>> {
    let root = sandbox.root_account().unwrap();
    Ok(root
        .create_subaccount(prefix)
        .initial_balance(NearToken::from_near(balance))
        .transact()
        .await?
        .result)
}

pub async fn create_ft(
    sandbox: &Worker<Sandbox>,
    name: &str,
    symbol: &str,
    decimals: u32,
    total_supply: Balance,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let mock_ft_contract_wasm =
        std::fs::read(MOCK_FT_CONTRACT_WASM).expect("Contract wasm not found");

    let ft_account = create_account(sandbox, symbol.to_lowercase().as_str(), 100).await?;
    let ft_contract = ft_account.deploy(&mock_ft_contract_wasm).await?.result;
    let result = ft_contract
        .call("new")
        .args_json(json!({
            "owner_id": ft_contract.id(),
            "total_supply": total_supply.to_string(),
            "metadata": {
                "spec": "ft-1.0.0".to_string(),
                "name": name.to_string(),
                "symbol": symbol.to_string(),
                "icon": None::<String>,
                "reference": None::<String>,
                "reference_hash": None::<String>,
                "decimals": decimals,
            }
        }))
        .transact()
        .await?;
    assert!(
        result.is_success(),
        "{:#?}",
        result.into_result().unwrap_err()
    );

    let result = ft_contract.view("ft_metadata").await?;
    let metadata: FungibleTokenMetadata = serde_json::from_slice(&result.result).unwrap();
    println!(
        "\n [LOG] FT metadata: {{ name: {}, symbol: {}, decimals: {} }}",
        metadata.name, metadata.symbol, metadata.decimals
    );

    Ok(ft_contract)
}

pub async fn storage_deposit(
    ft: &Contract,
    account: &Account,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = ft
        .call("storage_deposit")
        .args_json(json!({
            "account_id": account.id(),
            "registration_only": true
        }))
        .deposit(NearToken::from_millinear(1250))
        .transact()
        .await?;

    Ok(result)
}

pub async fn ft_transfer(
    ft: &Contract,
    sender: &Account,
    receiver: &Account,
    amount: Balance,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = sender
        .call(ft.id(), "ft_transfer")
        .args_json(json!({
            "receiver_id": receiver.id(),
            "amount": amount.to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    println!("\nResult: transfer {} {} FT {:?}", amount, ft.id(), result);

    Ok(result)
}

pub async fn deploy_mock_intents(
    sandbox: &Worker<Sandbox>,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let mock_intents_contract_wasm =
        std::fs::read(MOCK_INTENTS_CONTRACT_WASM).expect("Contract wasm not found");
    let mock_intents_account = create_account(sandbox, "intents", 100).await?;
    let mock_intents_contract = mock_intents_account
        .deploy(&mock_intents_contract_wasm)
        .await?
        .result;

    println!("Initializing solver mock intents contract...");
    let result = mock_intents_contract.call("new").transact().await?;
    println!("\nResult init: {:?}", result);

    Ok(mock_intents_contract)
}

pub async fn deploy_solver_registry(
    sandbox: &Worker<Sandbox>,
    intents_contract: &Contract,
    owner: &Account,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let solver_registry_contract_wasm =
        std::fs::read(SOLVER_REGISTRY_CONTRACT_WASM).expect("Contract wasm not found");
    let solver_registry_account = create_account(sandbox, "solver-registry", 100).await?;
    let solver_registry_contract = solver_registry_account
        .deploy(&solver_registry_contract_wasm)
        .await?
        .result;

    println!("Initializing solver registry contract...");
    let result = solver_registry_contract
        .call("new")
        .args_json(json!({
            "owner_id": owner.id(),
            "intents_contract_id": intents_contract.id(),
        }))
        .transact()
        .await?;
    println!("\nResult init: {:?}", result);

    Ok(solver_registry_contract)
}

pub async fn deposit_into_pool(
    solver_registry: &Contract,
    user: &Account,
    pool_id: u32,
    ft: &Contract,
    amount: Balance,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = user
        .call(ft.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": solver_registry.id(),
            "amount": amount.to_string(),
            "msg": json!({
                "DepositIntoPool": {
                    "pool_id": pool_id
                }
            }).to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    println!("\nResult: deposit token {:?}", result);

    Ok(result)
}
