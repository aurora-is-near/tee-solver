use near_sdk::serde_json::json;

mod common;

use common::constants::*;
use common::utils::*;

#[tokio::test]
async fn test_remove_compose_hash() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for remove compose hash...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, _bob, _mock_intents, solver_registry) =
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
