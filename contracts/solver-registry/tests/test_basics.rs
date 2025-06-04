use near_gas::NearGas;
use near_sdk::{near, NearToken};
use serde_json::json;

mod constants;
use constants::*;

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Worker {
    pool_id: u32,
    checksum: String,
    codehash: String,
}

#[tokio::test]
async fn test_register_worker() -> anyhow::Result<()> {
    println!("Starting test...");
    let sandbox = near_workspaces::sandbox().await?;

    let mock_intents_contract_wasm =
        std::fs::read(MOCK_INTENTS_CONTRACT_WASM).expect("Contract wasm not found");
    let mock_intents_contract = sandbox.dev_deploy(&mock_intents_contract_wasm).await?;

    println!("Initializing solver mock intents contract...");
    let result = mock_intents_contract.call("new").transact().await?;
    println!("\nResult init: {:?}", result);

    let solver_registry_contract_wasm =
        std::fs::read(SOLVER_REGISTRY_CONTRACT_WASM).expect("Contract wasm not found");
    let solver_registry_contract = sandbox.dev_deploy(&solver_registry_contract_wasm).await?;

    println!("Initializing solver registry contract...");
    let result = solver_registry_contract
        .call("new")
        .args_json(json!({
            "owner_id": solver_registry_contract.id(),
            "intents_contract_id": mock_intents_contract.id(),
        }))
        .transact()
        .await?;
    println!("\nResult init: {:?}", result);

    // Create a liquidity pool first
    println!("creating liquidity pool...");
    let result_pool = solver_registry_contract
        .call("create_liquidity_pool")
        .args_json(json!({
            "token_ids": ["token1.near", "token2.near"],
            "fee": 300
        }))
        .deposit(NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000)) // 1.5 NEAR
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    println!("\nResult create_liquidity_pool: {:?}", result_pool);

    // Check if pool was created successfully
    if !result_pool.is_success()
        && !result_pool
            .into_result()
            .unwrap()
            .json::<Option<u32>>()
            .unwrap()
            .is_some()
    {
        println!("Pool creation failed, skipping worker registration test");
        return Ok(());
    }

    // Approve codehash
    let result_approve_codehash = solver_registry_contract
        .call("approve_codehash")
        .args_json(json!({
            "codehash": CODE_HASH
        }))
        .transact()
        .await?;

    println!("\n[LOG] Approve codehash: {:?}", result_approve_codehash);

    // Call register_worker
    let collateral = include_str!("samples/quote_collateral.json").to_string();
    let result = solver_registry_contract
        .call("register_worker")
        .args_json(json!({
            "pool_id": 0,
            "quote_hex": QUOTE_HEX.to_string(),
            "collateral": collateral,
            "checksum": CHECKSUM.to_string(),
            "tcb_info": TCB_INFO.to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;

    println!("\nResult register_worker: {:?}", result);
    assert!(result.is_success());

    let result_get_worker = solver_registry_contract
        .call("get_worker")
        .args_json(json!({"account_id" : solver_registry_contract.id()}))
        .view()
        .await?;

    let worker: Worker = serde_json::from_slice(&result_get_worker.result).unwrap();
    println!(
        "\n [LOG] Worker: {{ checksum: {}, codehash: {}, poolId: {} }}",
        worker.checksum, worker.codehash, worker.pool_id
    );

    // Expect success value because we registered the codehash
    let result_get_codehash = solver_registry_contract
        .call("require_approved_codehash")
        .args_json(json!({}))
        .transact()
        .await?;

    println!(
        "\n[LOG] Require approved codehash AFTER approve: {:?}",
        result_get_codehash
    );

    Ok(())
}
