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

    // Setup test environment
    let (wnear, usdc, owner, alice, bob, mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Get pool info to verify creation
    let pool = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool: {{ token_ids: {:?}, amounts: {:?}, fee: {}, shares_total_supply: {:?} }}",
        pool.token_ids, pool.amounts, pool.fee, pool.shares_total_supply
    );

    // Approve codehash
    approve_codehash(&owner, &solver_registry).await?;

    // Register worker (Alice)
    println!("Registering worker (Alice)...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "Worker registration should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify worker registration
    let worker_info_option = get_worker_info(&solver_registry, &alice).await?;
    let worker_info = worker_info_option.expect("Alice should be registered as a worker");
    println!(
        "\n [LOG] Worker: {{ checksum: {}, codehash: {}, poolId: {} }}",
        worker_info.checksum, worker_info.codehash, worker_info.pool_id
    );

    // Create a funder account for token transfers
    let funder = create_account(&sandbox, "funder", 10).await?;

    // Register funder for NEP-141 tokens
    let _ = storage_deposit(&wnear, &funder).await?;
    let _ = storage_deposit(&usdc, &funder).await?;

    // Transfer some wNEAR and USDC to funder
    let _ = ft_transfer(
        &wnear,
        wnear.as_account(),
        &funder,
        NearToken::from_near(100).as_yoctonear(),
    )
    .await?;
    let _ = ft_transfer(&usdc, usdc.as_account(), &funder, 500_000_000).await?;

    // Deposit some 10 NEAR and 50 USDC into liquidity pool
    let _ = deposit_into_pool(
        &solver_registry,
        &funder,
        0,
        &wnear,
        NearToken::from_near(10).as_yoctonear(),
    )
    .await?;
    let _ = deposit_into_pool(&solver_registry, &funder, 0, &usdc, 50_000_000).await?;

    println!("Test passed: Worker registration and pool setup completed successfully");

    Ok(())
}
