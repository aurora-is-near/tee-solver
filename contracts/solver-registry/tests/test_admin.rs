use near_sdk::NearToken;
use near_sdk::serde_json::json;

mod common;

use common::constants::*;
use common::utils::*;

#[tokio::test]
async fn test_remove_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for remove compose hash...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, _alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, DEFAULT_WORKER_PING_TIMEOUT_MS).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash first
    println!("Approving compose hash...");
    approve_compose_hash(&owner, &solver_registry).await?;

    // Verify compose hash is approved
    let approved_hashes = get_approved_compose_hashes(&solver_registry).await?;
    assert_eq!(
        approved_hashes.len(),
        1,
        "Should have exactly one approved compose hash"
    );
    assert_eq!(
        approved_hashes[0], COMPOSE_HASH,
        "Approved hash should match COMPOSE_HASH"
    );

    // Remove compose hash as owner
    println!("Removing compose hash as owner...");
    remove_compose_hash(&owner, &solver_registry, COMPOSE_HASH).await?;

    // Verify compose hash is removed
    let approved_hashes_after = get_approved_compose_hashes(&solver_registry).await?;
    assert_eq!(
        approved_hashes_after.len(),
        0,
        "Should have no approved compose hashes after removal"
    );

    println!("Test passed: Owner can successfully remove compose hash");

    Ok(())
}

#[tokio::test]
async fn test_remove_compose_hash_with_non_owner() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for remove compose hash with non-owner...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, DEFAULT_WORKER_PING_TIMEOUT_MS).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash first
    approve_compose_hash(&owner, &solver_registry).await?;

    // Try to remove compose hash with non-owner (Alice)
    println!("Attempting to remove compose hash with non-owner...");
    let result = alice
        .call(solver_registry.id(), "remove_compose_hash")
        .args_json(json!({
            "compose_hash": COMPOSE_HASH
        }))
        .transact()
        .await?;

    // Removal should fail with non-owner
    assert!(
        !result.is_success(),
        "Compose hash removal should fail with non-owner"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    // Verify compose hash is still approved
    let approved_hashes = get_approved_compose_hashes(&solver_registry).await?;
    assert_eq!(
        approved_hashes.len(),
        1,
        "Compose hash should still be approved after failed removal attempt"
    );
    assert_eq!(
        approved_hashes[0], COMPOSE_HASH,
        "Approved hash should still match COMPOSE_HASH"
    );

    println!("Test passed: Only owner can remove compose hash");

    Ok(())
}

#[tokio::test]
async fn test_remove_nonexistent_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for remove nonexistent compose hash...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, _alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, DEFAULT_WORKER_PING_TIMEOUT_MS).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Try to remove a compose hash that was never approved
    println!("Attempting to remove nonexistent compose hash...");
    let nonexistent_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    let result = owner
        .call(solver_registry.id(), "remove_compose_hash")
        .args_json(json!({
            "compose_hash": nonexistent_hash
        }))
        .transact()
        .await?;

    // Removal should fail because hash doesn't exist
    assert!(
        !result.is_success(),
        "Removal of nonexistent compose hash should fail"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    // Verify no compose hashes are approved
    let approved_hashes = get_approved_compose_hashes(&solver_registry).await?;
    assert_eq!(
        approved_hashes.len(),
        0,
        "Should have no approved compose hashes"
    );

    println!("Test passed: Removing nonexistent compose hash fails correctly");

    Ok(())
}

#[tokio::test]
async fn test_upgrade_contract() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for contract upgrade...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, _alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, DEFAULT_WORKER_PING_TIMEOUT_MS).await?;

    // Create a liquidity pool to have some state
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Verify pool exists before upgrade
    let pool_len_before = get_pool_len(&solver_registry).await?;
    assert_eq!(pool_len_before, 1, "Pool should exist before upgrade");

    // Read the contract WASM file
    let contract_wasm = std::fs::read(common::utils::SOLVER_REGISTRY_CONTRACT_WASM)
        .expect("Contract WASM file not found");

    println!("Calling upgrade function...");
    // Call upgrade with the contract code
    // The upgrade function reads from env::input(), so we pass the WASM bytes as raw args
    let result = owner
        .call(solver_registry.id(), "upgrade")
        .args(contract_wasm)
        .deposit(NearToken::from_yoctonear(0))
        .max_gas()
        .transact()
        .await?;

    assert!(
        result.is_success(),
        "Upgrade should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    println!("Upgrade completed successfully");

    // Verify the contract still works after upgrade
    // Check that we can still query pools
    let pool_len_after = get_pool_len(&solver_registry).await?;
    assert_eq!(
        pool_len_after, pool_len_before,
        "Pool count should remain the same after upgrade"
    );

    // Verify we can still query approved compose hashes
    let approved_hashes = get_approved_compose_hashes(&solver_registry).await?;
    println!(
        "Approved compose hashes after upgrade: {:?}",
        approved_hashes
    );

    // Verify we can still get worker ping timeout
    let timeout = get_worker_ping_timeout_ms(&solver_registry).await?;
    assert_eq!(
        timeout, DEFAULT_WORKER_PING_TIMEOUT_MS,
        "Worker ping timeout should remain the same after upgrade"
    );

    println!("Test passed: Contract upgrade completed successfully and state is preserved");

    Ok(())
}

#[tokio::test]
async fn test_upgrade_contract_with_non_owner() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for contract upgrade with non-owner...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, _owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, DEFAULT_WORKER_PING_TIMEOUT_MS).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Read the contract WASM file
    let contract_wasm = std::fs::read(common::utils::SOLVER_REGISTRY_CONTRACT_WASM)
        .expect("Contract WASM file not found");

    // Try to upgrade with non-owner (Alice)
    println!("Attempting to upgrade contract with non-owner...");
    let result = alice
        .call(solver_registry.id(), "upgrade")
        .args(contract_wasm)
        .deposit(NearToken::from_yoctonear(0))
        .max_gas()
        .transact()
        .await?;

    // Upgrade should fail with non-owner
    assert!(!result.is_success(), "Upgrade should fail with non-owner");

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    // Verify contract still works (upgrade didn't happen)
    let pool_len = get_pool_len(&solver_registry).await?;
    assert_eq!(
        pool_len, 1,
        "Pool should still exist after failed upgrade attempt"
    );

    println!("Test passed: Only owner can upgrade the contract");

    Ok(())
}
