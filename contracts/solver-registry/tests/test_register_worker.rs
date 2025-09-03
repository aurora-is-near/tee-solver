use near_gas::NearGas;
use near_sdk::NearToken;
use serde_json::json;

mod common;

use common::constants::*;
use common::utils::*;

#[tokio::test]
async fn test_register_one_worker() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Get pool info to verify creation
    let pool = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool: {{ token_ids: {:?}, amounts: {:?}, fee: {}, shares_total_supply: {:?} }}",
        pool.token_ids, pool.amounts, pool.fee, pool.shares_total_supply
    );

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

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
        "\n [LOG] Worker: {{ checksum: {}, compose_hash: {}, pool_id: {} }}",
        worker_info.checksum, worker_info.compose_hash, worker_info.pool_id
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

#[tokio::test]
async fn test_worker_registration_with_invalid_tee_data() -> Result<(), Box<dyn std::error::Error>>
{
    println!("Starting test for worker registration with invalid TEE data...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Try to register worker with invalid quote hex (empty string)
    println!("Attempting to register worker with invalid quote hex...");
    let result = register_worker(
        &alice,
        &solver_registry,
        0,
        "", // Invalid empty quote hex
        QUOTE_COLLATERAL_ALICE,
        CHECKSUM_ALICE,
        TCB_INFO_ALICE,
    )
    .await?;

    // Registration should fail with invalid TEE data
    assert!(
        !result.is_success(),
        "Worker registration should fail with invalid quote hex"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {:?}", error);

    println!("Test passed: Worker registration properly validates TEE data");

    Ok(())
}

#[tokio::test]
async fn test_worker_registration_requires_sufficient_deposit(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker registration requires sufficient deposit...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Try to register worker with insufficient deposit (0 yoctoNEAR)
    println!("Attempting to register worker with insufficient deposit...");
    let result = alice
        .call(solver_registry.id(), "register_worker")
        .args_json(json!({
            "pool_id": 0,
            "quote_hex": QUOTE_HEX_ALICE.to_string(),
            "collateral": QUOTE_COLLATERAL_ALICE.to_string(),
            "checksum": CHECKSUM_ALICE.to_string(),
            "tcb_info": TCB_INFO_ALICE.to_string()
        }))
        .deposit(NearToken::from_yoctonear(0)) // No deposit
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;

    // Registration should fail with insufficient deposit
    assert!(
        !result.is_success(),
        "Worker registration should fail with insufficient deposit"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {:?}", error);

    println!("Test passed: Worker registration requires sufficient deposit");

    Ok(())
}

#[tokio::test]
async fn test_worker_registration_without_compose_hash_approval(
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker registration without compose hash approval...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, _owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Try to register worker without approving compose hash first
    println!("Attempting to register worker without compose hash approval...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;

    // Registration should fail without compose hash approval
    assert!(
        !result.is_success(),
        "Worker registration should fail without compose hash approval"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {:?}", error);

    println!("Test passed: Worker registration requires compose hash approval");

    Ok(())
}

#[tokio::test]
async fn test_approve_compose_hash_with_non_owner() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for compose hash approval with non-owner...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, _owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Try to approve compose hash with non-owner (Alice)
    println!("Attempting to approve compose hash with non-owner...");
    let result = alice
        .call(solver_registry.id(), "approve_compose_hash")
        .args_json(json!({
            "compose_hash": COMPOSE_HASH
        }))
        .transact()
        .await?;

    // Compose hash approval should fail with non-owner
    assert!(
        !result.is_success(),
        "Compose hash approval should fail with non-owner"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {:?}", error);

    println!("Test passed: Only owner can approve compose hash");

    Ok(())
}

#[tokio::test]
async fn test_worker_registration_with_invalid_pool_id() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker registration with invalid pool ID...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool (pool_id = 0)
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Try to register worker with non-existent pool ID
    println!("Attempting to register worker with non-existent pool ID...");
    let result = register_worker_alice(&alice, &solver_registry, 999).await?;

    // Registration should fail with invalid pool ID
    assert!(
        !result.is_success(),
        "Worker registration should fail with invalid pool ID"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {:?}", error);

    println!("Test passed: Worker registration validates pool ID");

    Ok(())
}

#[tokio::test]
async fn test_multiple_pools_worker_registration() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for multiple pools worker registration...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create multiple liquidity pools
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Create a second pool with different tokens (using same tokens for simplicity)
    let result = solver_registry
        .call("create_liquidity_pool")
        .args_json(json!({
            "token_ids": [wnear.id(), usdc.id()],
            "fee": 500
        }))
        .deposit(NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000)) // 1.5 NEAR
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(result.is_success(), "Second pool creation should succeed");

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Register Alice as worker for pool 0
    println!("Registering Alice as worker for pool 0...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "Alice should be able to register for pool 0"
    );

    // Register Bob as worker for pool 1
    println!("Registering Bob as worker for pool 1...");
    let result = register_worker_bob(&bob, &solver_registry, 1).await?;
    assert!(
        result.is_success(),
        "Bob should be able to register for pool 1"
    );

    // Verify both workers are registered for different pools
    let alice_worker = get_worker_info(&solver_registry, &alice).await?;
    let bob_worker = get_worker_info(&solver_registry, &bob).await?;

    assert!(
        alice_worker.is_some(),
        "Alice should be registered as a worker"
    );
    assert!(bob_worker.is_some(), "Bob should be registered as a worker");

    let alice_info = alice_worker.unwrap();
    let bob_info = bob_worker.unwrap();

    assert_eq!(
        alice_info.pool_id, 0,
        "Alice should be registered for pool 0"
    );
    assert_eq!(bob_info.pool_id, 1, "Bob should be registered for pool 1");

    // Verify both workers can ping their respective pools
    let result = ping_worker(&alice, &solver_registry).await?;
    assert!(result.is_success(), "Alice should be able to ping pool 0");

    let result = ping_worker(&bob, &solver_registry).await?;
    assert!(result.is_success(), "Bob should be able to ping pool 1");

    println!("Test passed: Multiple pools can have different workers");

    Ok(())
}

#[tokio::test]
async fn test_worker_registration_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker registration edge cases...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Test 1: Register with valid pool ID
    println!("Testing registration with valid pool ID...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "Registration with valid pool ID should succeed"
    );

    // Test 2: Try to register same worker again (should fail)
    println!("Testing duplicate worker registration...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        !result.is_success(),
        "Duplicate worker registration should fail"
    );

    // Test 3: Try to register with very large pool ID
    println!("Testing registration with very large pool ID...");
    let result = register_worker_bob(&bob, &solver_registry, u32::MAX).await?;
    assert!(
        !result.is_success(),
        "Registration with very large pool ID should fail"
    );

    // Test 4: Try to register with empty TEE parameters
    println!("Testing registration with empty TEE parameters...");
    let result = register_worker(&bob, &solver_registry, 0, "", "", "", "").await?;
    assert!(
        !result.is_success(),
        "Registration with empty TEE parameters should fail"
    );

    println!("Test passed: Worker registration edge cases are properly handled");

    Ok(())
}
