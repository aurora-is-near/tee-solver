use std::str::FromStr;

use near_gas::NearGas;
use near_sdk::NearToken;
use near_workspaces::types::SecretKey;
use serde_json::json;

mod constants;
mod utils;

use constants::*;
use utils::*;

#[tokio::test]
async fn test_register_worker() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test...");
    let sandbox = near_workspaces::sandbox().await?;

    println!("Deploying wNEAR contract...");
    let wnear = create_ft(
        &sandbox,
        "Wrapped NEAR",
        "wNEAR",
        24,
        NearToken::from_near(1_000_000_000).as_yoctonear(), // 1B
    )
    .await?;

    println!("Deploying USDC contract...");
    let usdc = create_ft(
        &sandbox,
        "USD Coin",
        "USDC",
        6,
        10_000_000_000_000_000, // 10B
    )
    .await?;

    let owner = create_account(&sandbox, "owner", 10).await?;
    let funder = create_account(&sandbox, "funder", 10).await?;
    let alice = create_account_with_secret_key(
        &sandbox,
        "worker",
        10,
        SecretKey::from_str(SECRET_KEY_ALICE).unwrap(),
    )
    .await?;

    // Reigster accounts for NEP-141 tokens
    let _ = storage_deposit(&wnear, &funder).await?;
    let _ = storage_deposit(&usdc, &funder).await?;

    println!("Deploying mockd intents contract...");
    let mock_intents = deploy_mock_intents(&sandbox).await?;

    println!("Deploying Solver Registry contract...");
    let solver_registry =
        deploy_solver_registry(&sandbox, &mock_intents, &owner, 10 * 60 * 1000).await?;

    // Reigster contracts for NEP-141 tokens
    let _ = storage_deposit(&wnear, mock_intents.as_account()).await?;
    let _ = storage_deposit(&usdc, mock_intents.as_account()).await?;

    let _ = storage_deposit(&wnear, solver_registry.as_account()).await?;
    let _ = storage_deposit(&usdc, solver_registry.as_account()).await?;

    // Create a liquidity pool first
    println!("Creating liquidity pool...");
    let result = solver_registry
        .call("create_liquidity_pool")
        .args_json(json!({
            "token_ids": [wnear.id(), usdc.id()],
            "fee": 300
        }))
        .deposit(NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000)) // 1.5 NEAR
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(
        result.is_success(),
        "{:#?}",
        result.into_result().unwrap_err()
    );

    let result = solver_registry
        .view("get_pool")
        .args_json(json!({"pool_id" : 0}))
        .await?;
    let pool: PoolInfo = serde_json::from_slice(&result.result).unwrap();
    println!(
        "\n [LOG] Pool: {{ token_ids: {:?}, amounts: {:?}, fee: {}, shares_total_supply: {:?} }}",
        pool.token_ids, pool.amounts, pool.fee, pool.shares_total_supply
    );

    // Approve codehash by owner
    let result = owner
        .call(solver_registry.id(), "approve_codehash")
        .args_json(json!({
            "codehash": CODE_HASH
        }))
        .transact()
        .await?;
    assert!(
        result.is_success(),
        "{:#?}",
        result.into_result().unwrap_err()
    );

    // Register worker
    let collateral = include_str!("samples/alice/quote_collateral.json").to_string();
    let result = alice
        .call(solver_registry.id(), "register_worker")
        .args_json(json!({
            "pool_id": 0,
            "quote_hex": QUOTE_HEX_ALICE.to_string(),
            "collateral": collateral,
            "checksum": CHECKSUM_ALICE.to_string(),
            "tcb_info": TCB_INFO_ALICE.to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(
        result.is_success(),
        "{:#?}",
        result.into_result().unwrap_err()
    );

    let result_get_worker = solver_registry
        .view("get_worker")
        .args_json(json!({"account_id" : alice.id()}))
        .await?;

    let worker_info: WorkerInfo = serde_json::from_slice(&result_get_worker.result).unwrap();
    println!(
        "\n [LOG] Worker: {{ checksum: {}, codehash: {}, poolId: {} }}",
        worker_info.checksum, worker_info.codehash, worker_info.pool_id
    );

    // Transfer some wNEAR and USDC to Alice
    let _ = ft_transfer(
        &wnear,
        wnear.as_account(),
        &funder,
        NearToken::from_near(100).as_yoctonear(),
    )
    .await?;
    let _ = ft_transfer(&usdc, usdc.as_account(), &funder, 500_000_000).await?;

    // Deposint some 10 NEAR and 50 USDC into liquidity pool
    let _ = deposit_into_pool(
        &solver_registry,
        &funder,
        0,
        &wnear,
        NearToken::from_near(10).as_yoctonear(),
    )
    .await?;
    let _ = deposit_into_pool(&solver_registry, &funder, 0, &usdc, 50_000_000).await?;

    Ok(())
}
