#![allow(clippy::too_many_lines)]
mod common;

use common::utils::*;

#[tokio::test]
async fn test_only_one_active_worker_per_pool() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for one active worker per pool...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, bob, mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Get pool account ID before worker registration
    let pool_account_id = get_pool_account_id(&solver_registry, 0);
    println!("\n [LOG] Pool Account ID: {pool_account_id}");

    // Verify no public keys exist for the pool initially
    let initial_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert!(
        initial_public_keys.is_empty(),
        "Pool should have no public keys initially"
    );
    println!(
        "\n [LOG] Initial public keys count: {}",
        initial_public_keys.len()
    );

    // Register first worker (Alice)
    println!("Registering first worker (Alice)...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "First worker registration should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify first worker is registered
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    let worker = alice_worker_option.expect("Alice should be registered as a worker");
    println!(
        "\n [LOG] First Worker (Alice): {{ checksum: {}, compose_hash: {}, pool_id: {} }}",
        worker.checksum, worker.compose_hash, worker.pool_id
    );

    // Verify that Alice's public key is now in the mock-intents contract for the pool
    let alice_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        alice_public_keys.len(),
        1,
        "Pool should have exactly one public key after Alice's registration"
    );
    assert!(
        alice_public_keys.contains(&worker.public_key),
        "Pool should contain Alice's public key"
    );
    println!(
        "\n [LOG] Public keys after Alice registration: {}",
        alice_public_keys.len()
    );

    // Try to register second worker (Bob) for the same pool - this should fail
    println!("Attempting to register second worker (Bob) for the same pool...");
    let result = register_worker_bob(&bob, &solver_registry, 0).await?;

    // The second registration should fail with "Only one active worker is allowed per pool"
    assert!(
        !result.is_success(),
        "Second worker registration should fail, but it succeeded"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    // Verify that Bob is not registered as a worker
    let bob_worker_option = get_worker_info(&solver_registry, &bob).await?;
    assert!(
        bob_worker_option.is_none(),
        "Bob should not be registered as a worker"
    );

    // Verify that Alice is still the only worker for the pool
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    let alice_worker = alice_worker_option.expect("Alice should be registered as a worker");
    assert_eq!(
        alice_worker.pool_id, 0,
        "Alice should still be registered for pool 0"
    );

    // Verify that the pool still only contains Alice's public key
    let final_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        final_public_keys.len(),
        1,
        "Pool should still have exactly one public key"
    );
    assert!(
        final_public_keys.contains(&alice_worker.public_key),
        "Pool should still contain only Alice's public key"
    );
    println!(
        "\n [LOG] Final public keys count: {}",
        final_public_keys.len()
    );

    println!("Test passed: Only one active worker is allowed per pool");

    Ok(())
}

#[tokio::test]
async fn test_worker_ping_functionality() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker ping functionality...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, bob, mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Get pool account ID
    let pool_account_id = get_pool_account_id(&solver_registry, 0);
    println!("\n [LOG] Pool Account ID: {pool_account_id}");

    // Verify no public keys exist for the pool initially
    let initial_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert!(
        initial_public_keys.is_empty(),
        "Pool should have no public keys initially"
    );
    println!(
        "\n [LOG] Initial public keys count: {}",
        initial_public_keys.len()
    );

    // Register worker (Alice)
    println!("Registering worker (Alice)...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "Worker registration should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify that Alice's public key is now in the mock-intents contract for the pool
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    let alice_worker = alice_worker_option.expect("Alice should be registered as a worker");
    let alice_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        alice_public_keys.len(),
        1,
        "Pool should have exactly one public key after Alice's registration"
    );
    assert!(
        alice_public_keys.contains(&alice_worker.public_key),
        "Pool should contain Alice's public key"
    );
    println!(
        "\n [LOG] Public keys after Alice registration: {}",
        alice_public_keys.len()
    );

    // Get initial pool state
    let pool_initial = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Initial Pool State: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_initial.worker_id, pool_initial.last_ping_timestamp_ms
    );

    // Worker pings to maintain active status
    println!("Worker (Alice) pinging to maintain active status...");
    let result = ping_worker(&alice, &solver_registry).await?;
    assert!(
        result.is_success(),
        "Worker ping should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Get pool state after ping
    let pool_after_ping = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool State After Ping: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_after_ping.worker_id, pool_after_ping.last_ping_timestamp_ms
    );

    // Verify that the ping timestamp was updated
    assert!(
        pool_after_ping.last_ping_timestamp_ms > pool_initial.last_ping_timestamp_ms,
        "Ping timestamp should be updated"
    );

    // Verify that Alice's public key is still the only one in the pool
    let public_keys_after_ping = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_after_ping.len(),
        1,
        "Pool should still have exactly one public key after ping"
    );
    assert!(
        public_keys_after_ping.contains(&alice_worker.public_key),
        "Pool should still contain Alice's public key after ping"
    );

    // Test that only the registered worker can ping
    println!("Testing that only the registered worker can ping...");
    let result = ping_worker(&bob, &solver_registry).await?;

    // Bob should not be able to ping since he's not a registered worker
    assert!(
        !result.is_success(),
        "Non-registered worker should not be able to ping"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    // Verify that Bob's public key is not in the pool
    let public_keys_after_bob_ping_attempt =
        get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_after_bob_ping_attempt.len(),
        1,
        "Pool should still have exactly one public key after Bob's failed ping attempt"
    );
    assert!(
        public_keys_after_bob_ping_attempt.contains(&alice_worker.public_key),
        "Pool should still contain only Alice's public key after Bob's failed ping attempt"
    );

    // Verify that Alice can still ping successfully
    println!("Worker (Alice) pinging again...");
    let result = ping_worker(&alice, &solver_registry).await?;
    assert!(
        result.is_success(),
        "Registered worker should still be able to ping: {:#?}",
        result.into_result().unwrap_err()
    );

    // Get final pool state
    let pool_final = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Final Pool State: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_final.worker_id, pool_final.last_ping_timestamp_ms
    );

    // Verify that the final ping timestamp is greater than the previous one
    assert!(
        pool_final.last_ping_timestamp_ms > pool_after_ping.last_ping_timestamp_ms,
        "Final ping timestamp should be greater than the previous ping"
    );

    // Verify that Alice's public key is still the only one in the pool
    let final_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        final_public_keys.len(),
        1,
        "Pool should still have exactly one public key at the end"
    );
    assert!(
        final_public_keys.contains(&alice_worker.public_key),
        "Pool should still contain only Alice's public key at the end"
    );
    println!(
        "\n [LOG] Final public keys count: {}",
        final_public_keys.len()
    );

    println!("Test passed: Worker ping functionality works correctly");

    Ok(())
}

#[tokio::test]
async fn test_worker_replacement_after_timeout() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker replacement after timeout...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment with shorter timeout for testing
    let (wnear, usdc, owner, alice, bob, mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 5 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Get pool account ID
    let pool_account_id = get_pool_account_id(&solver_registry, 0);
    println!("\n [LOG] Pool Account ID: {pool_account_id}");

    // Verify no public keys exist for the pool initially
    let initial_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert!(
        initial_public_keys.is_empty(),
        "Pool should have no public keys initially"
    );
    println!(
        "\n [LOG] Initial public keys count: {}",
        initial_public_keys.len()
    );

    // Register first worker (Alice)
    println!("Registering first worker (Alice)...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "First worker registration should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify first worker is registered
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    let worker = alice_worker_option.expect("Alice should be registered as a worker");
    println!(
        "\n [LOG] First Worker (Alice): {{ checksum: {}, compose_hash: {}, pool_id: {} }}",
        worker.checksum, worker.compose_hash, worker.pool_id
    );

    // Try to register second worker (Bob) for the same pool - this should fail
    println!("Attempting to register second worker (Bob) while Alice is active...");
    let result = register_worker_bob(&bob, &solver_registry, 0).await?;

    // The second registration should fail while Alice is active
    assert!(
        !result.is_success(),
        "Second worker registration should fail while Alice is active"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    // Verify that the pool still only contains Alice's public key
    let public_keys_after_failed_bob_registration =
        get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_after_failed_bob_registration.len(),
        1,
        "Pool should still have exactly one public key after failed Bob registration"
    );
    assert!(
        public_keys_after_failed_bob_registration.contains(&worker.public_key),
        "Pool should still contain only Alice's public key after failed Bob registration"
    );

    // Wait for the worker timeout (5 seconds) to allow worker replacement
    wait_for_worker_timeout(5).await;

    // Check pool info to see the current worker status after timeout
    let pool = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool after timeout: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool.worker_id, pool.last_ping_timestamp_ms
    );

    // Now try to register Bob as the new worker - this should succeed
    println!("Attempting to register Bob as the new worker after timeout...");
    let result = register_worker_bob(&bob, &solver_registry, 0).await?;

    // The registration should now succeed since Alice has timed out
    assert!(
        result.is_success(),
        "Bob should be able to register after Alice's timeout: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify that Bob is now registered as the worker
    let bob_worker_option = get_worker_info(&solver_registry, &bob).await?;
    let bob_worker = bob_worker_option.expect("Bob should be registered as a worker");
    println!(
        "\n [LOG] New Worker (Bob): {{ checksum: {}, compose_hash: {}, pool_id: {} }}",
        bob_worker.checksum, bob_worker.compose_hash, bob_worker.pool_id
    );
    assert_eq!(bob_worker.pool_id, 0, "Bob should be registered for pool 0");

    // Verify that the pool now contains Bob's public key instead of Alice's
    let public_keys_after_bob_registration =
        get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_after_bob_registration.len(),
        1,
        "Pool should have exactly one public key after Bob's registration"
    );
    assert!(
        public_keys_after_bob_registration.contains(&bob_worker.public_key),
        "Pool should contain Bob's public key after registration"
    );
    assert!(
        !public_keys_after_bob_registration.contains(&worker.public_key),
        "Pool should no longer contain Alice's public key after Bob's registration"
    );
    println!(
        "\n [LOG] Public keys after Bob registration: {}",
        public_keys_after_bob_registration.len()
    );

    // Verify that Alice is still registered although it's not active
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    assert!(
        alice_worker_option.is_none(),
        "Alice should has been removed"
    );

    // Verify that Bob is now the active worker for the pool
    let pool_final = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Final Pool State: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_final.worker_id, pool_final.last_ping_timestamp_ms
    );
    assert_eq!(
        pool_final.worker_id,
        Some(bob.id().clone()),
        "Bob should be the active worker for the pool"
    );

    println!("Test passed: Worker replacement after timeout works correctly");

    Ok(())
}

#[tokio::test]
async fn test_worker_cannot_register_while_active_worker_is_pinging()
-> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker cannot register while active worker is pinging...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment with short timeout for testing (5 seconds)
    let (wnear, usdc, owner, alice, bob, mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 5 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Register Alice as the first worker
    println!("Registering Alice as the first worker...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "Alice's registration should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify Alice is registered
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    let alice_worker = alice_worker_option.expect("Alice should be registered as a worker");
    println!(
        "\n [LOG] Alice registered: {{ checksum: {}, compose_hash: {}, pool_id: {} }}",
        alice_worker.checksum, alice_worker.compose_hash, alice_worker.pool_id
    );

    // Verify that Alice's public key is now in the mock-intents contract for the pool
    let pool_account_id = get_pool_account_id(&solver_registry, 0);
    let alice_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        alice_public_keys.len(),
        1,
        "Pool should have exactly one public key after Alice's registration"
    );
    assert!(
        alice_public_keys.contains(&alice_worker.public_key),
        "Pool should contain Alice's public key"
    );
    println!(
        "\n [LOG] Public keys after Alice registration: {}",
        alice_public_keys.len()
    );

    // Get initial pool state
    let pool_initial = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Initial Pool State: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_initial.worker_id, pool_initial.last_ping_timestamp_ms
    );

    // Alice pings once to establish her initial timestamp
    println!("Alice pinging once to establish initial timestamp...");
    let result = ping_worker(&alice, &solver_registry).await?;
    assert!(
        result.is_success(),
        "Alice's initial ping should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Get pool state after Alice's initial ping
    let pool_after_initial_ping = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool after Alice's initial ping: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_after_initial_ping.worker_id, pool_after_initial_ping.last_ping_timestamp_ms
    );

    // Verify that the ping timestamp was updated
    assert!(
        pool_after_initial_ping.last_ping_timestamp_ms > pool_initial.last_ping_timestamp_ms,
        "Ping timestamp should be updated after initial ping"
    );

    // Verify that Alice's public key is still the only one in the pool
    let pool_account_id = get_pool_account_id(&solver_registry, 0);
    let public_keys_after_initial_ping =
        get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_after_initial_ping.len(),
        1,
        "Pool should still have exactly one public key after initial ping"
    );
    assert!(
        public_keys_after_initial_ping.contains(&alice_worker.public_key),
        "Pool should still contain Alice's public key after initial ping"
    );

    // Now Alice stops pinging and we wait for the timeout
    println!("Alice stops pinging. Waiting for timeout (5 seconds)...");
    wait_for_worker_timeout(5).await;

    // Get pool state after timeout
    let pool_after_timeout = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool after timeout: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_after_timeout.worker_id, pool_after_timeout.last_ping_timestamp_ms
    );

    // Verify that Alice is still technically the worker (but inactive)
    assert_eq!(
        pool_after_timeout.worker_id,
        Some(alice.id().clone()),
        "Alice should still be the worker in the pool (but inactive)"
    );

    // Alice demonstrates active pinging to maintain her status
    demonstrate_active_worker_pinging(&alice, &solver_registry, 3, 1000).await?;

    // Get pool state after Alice's active pinging
    let pool_after_active_pinging = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool after Alice's active pinging: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_after_active_pinging.worker_id, pool_after_active_pinging.last_ping_timestamp_ms
    );

    // Now try to register Bob while Alice is actively pinging - this should fail
    println!("Attempting to register Bob while Alice is actively pinging...");
    let result = register_worker_bob(&bob, &solver_registry, 0).await?;

    // Bob's registration should fail because Alice is actively pinging
    assert!(
        !result.is_success(),
        "Bob should not be able to register while Alice is actively pinging"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    // Verify that Bob's public key is not in the pool
    let public_keys_after_failed_bob_registration =
        get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_after_failed_bob_registration.len(),
        1,
        "Pool should still have exactly one public key after failed Bob registration"
    );
    assert!(
        public_keys_after_failed_bob_registration.contains(&alice_worker.public_key),
        "Pool should still contain only Alice's public key after failed Bob registration"
    );

    // Verify that Bob is not registered as a worker
    let bob_worker_option = get_worker_info(&solver_registry, &bob).await?;
    assert!(
        bob_worker_option.is_none(),
        "Bob should not be registered as a worker"
    );

    // Verify that Alice is still the active worker
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    let alice_worker_final =
        alice_worker_option.expect("Alice should still be registered as a worker");
    assert_eq!(
        alice_worker_final.pool_id, 0,
        "Alice should still be registered for pool 0"
    );

    // Verify that Alice is still the active worker for the pool
    let pool_final = get_pool_info(&solver_registry, 0).await?;
    assert_eq!(
        pool_final.worker_id,
        Some(alice.id().clone()),
        "Alice should still be the active worker for the pool"
    );

    // Verify that Alice's public key is still the only one in the pool
    let public_keys_final = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_final.len(),
        1,
        "Pool should still have exactly one public key at the end"
    );
    assert!(
        public_keys_final.contains(&alice_worker.public_key),
        "Pool should still contain only Alice's public key at the end"
    );

    // Alice pings one more time to demonstrate she's still active
    println!("Alice pinging one more time to demonstrate continued activity...");
    let result = ping_worker(&alice, &solver_registry).await?;
    assert!(
        result.is_success(),
        "Alice should still be able to ping: {:#?}",
        result.into_result().unwrap_err()
    );

    // Get final pool state
    let pool_final_after_ping = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Final Pool State: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_final_after_ping.worker_id, pool_final_after_ping.last_ping_timestamp_ms
    );

    // Verify that the final ping timestamp is greater than the previous one
    assert!(
        pool_final_after_ping.last_ping_timestamp_ms
            > pool_after_active_pinging.last_ping_timestamp_ms,
        "Final ping timestamp should be greater than the previous ping"
    );

    // Verify that Alice's public key is still the only one in the pool after final ping
    let final_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        final_public_keys.len(),
        1,
        "Pool should still have exactly one public key after final ping"
    );
    assert!(
        final_public_keys.contains(&alice_worker.public_key),
        "Pool should still contain only Alice's public key after final ping"
    );
    println!(
        "\n [LOG] Final public keys count: {}",
        final_public_keys.len()
    );

    println!("Test passed: Worker cannot register while active worker is pinging");

    Ok(())
}

#[tokio::test]
async fn test_worker_can_register_after_inactive_worker_timeout()
-> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker can register after inactive worker timeout...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment with short timeout for testing (5 seconds)
    let (wnear, usdc, owner, alice, bob, mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 5 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Register Alice as the first worker
    println!("Registering Alice as the first worker...");
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "Alice's registration should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify Alice is registered
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    let alice_worker = alice_worker_option.expect("Alice should be registered as a worker");
    println!(
        "\n [LOG] Alice registered: {{ checksum: {}, compose_hash: {}, pool_id: {} }}",
        alice_worker.checksum, alice_worker.compose_hash, alice_worker.pool_id
    );

    // Get initial pool state
    let pool_initial = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Initial Pool State: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_initial.worker_id, pool_initial.last_ping_timestamp_ms
    );

    // Alice pings once to establish her initial timestamp
    println!("Alice pinging once to establish initial timestamp...");
    let result = ping_worker(&alice, &solver_registry).await?;
    assert!(
        result.is_success(),
        "Alice's initial ping should succeed: {:#?}",
        result.into_result().unwrap_err()
    );

    // Get pool state after Alice's initial ping
    let pool_after_initial_ping = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool after Alice's initial ping: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_after_initial_ping.worker_id, pool_after_initial_ping.last_ping_timestamp_ms
    );

    // Verify that the ping timestamp was updated
    assert!(
        pool_after_initial_ping.last_ping_timestamp_ms > pool_initial.last_ping_timestamp_ms,
        "Ping timestamp should be updated after initial ping"
    );

    // Now Alice stops pinging and we wait for the timeout
    println!("Alice stops pinging. Waiting for timeout (5 seconds)...");
    wait_for_worker_timeout(5).await;

    // Get pool state after timeout
    let pool_after_timeout = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Pool after timeout: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_after_timeout.worker_id, pool_after_timeout.last_ping_timestamp_ms
    );

    // Verify that Alice is still technically the worker (but inactive)
    assert_eq!(
        pool_after_timeout.worker_id,
        Some(alice.id().clone()),
        "Alice should still be the worker in the pool (but inactive)"
    );

    // Now try to register Bob - this should succeed because Alice hasn't pinged after timeout
    println!("Attempting to register Bob after Alice's timeout...");
    let result = register_worker_bob(&bob, &solver_registry, 0).await?;

    // Bob's registration should succeed because Alice is inactive
    assert!(
        result.is_success(),
        "Bob should be able to register after Alice's timeout: {:#?}",
        result.into_result().unwrap_err()
    );

    // Verify that Bob is now registered as the worker
    let bob_worker_option = get_worker_info(&solver_registry, &bob).await?;
    let bob_worker = bob_worker_option.expect("Bob should be registered as a worker");
    println!(
        "\n [LOG] Bob registered: {{ checksum: {}, compose_hash: {}, pool_id: {} }}",
        bob_worker.checksum, bob_worker.compose_hash, bob_worker.pool_id
    );
    assert_eq!(bob_worker.pool_id, 0, "Bob should be registered for pool 0");

    // Verify that the pool now contains Bob's public key instead of Alice's
    let pool_account_id = get_pool_account_id(&solver_registry, 0);
    let public_keys_after_bob_registration =
        get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_after_bob_registration.len(),
        1,
        "Pool should have exactly one public key after Bob's registration"
    );
    assert!(
        public_keys_after_bob_registration.contains(&bob_worker.public_key),
        "Pool should contain Bob's public key after registration"
    );
    assert!(
        !public_keys_after_bob_registration.contains(&alice_worker.public_key),
        "Pool should no longer contain Alice's public key after Bob's registration"
    );
    println!(
        "\n [LOG] Public keys after Bob registration: {}",
        public_keys_after_bob_registration.len()
    );

    // Verify that Alice is still registered although it's not active
    let alice_worker_option = get_worker_info(&solver_registry, &alice).await?;
    assert!(
        alice_worker_option.is_none(),
        "Alice should has been removed"
    );

    // Verify that Bob is now the active worker for the pool
    let pool_final = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Final Pool State: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_final.worker_id, pool_final.last_ping_timestamp_ms
    );
    assert_eq!(
        pool_final.worker_id,
        Some(bob.id().clone()),
        "Bob should be the active worker for the pool"
    );

    // Verify that Bob can ping successfully
    println!("Bob pinging to demonstrate he's the active worker...");
    let result = ping_worker(&bob, &solver_registry).await?;
    assert!(
        result.is_success(),
        "Bob should be able to ping as the active worker: {:#?}",
        result.into_result().unwrap_err()
    );

    // Get final pool state after Bob's ping
    let pool_final_after_bob_ping = get_pool_info(&solver_registry, 0).await?;
    println!(
        "\n [LOG] Final Pool State after Bob's ping: {{ worker_id: {:?}, last_ping_timestamp_ms: {} }}",
        pool_final_after_bob_ping.worker_id, pool_final_after_bob_ping.last_ping_timestamp_ms
    );

    // Verify that Bob's ping updated the timestamp
    assert!(
        pool_final_after_bob_ping.last_ping_timestamp_ms > pool_final.last_ping_timestamp_ms,
        "Bob's ping should update the timestamp"
    );

    // Verify that the pool still contains Bob's public key after his ping
    let public_keys_after_bob_ping = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert_eq!(
        public_keys_after_bob_ping.len(),
        1,
        "Pool should still have exactly one public key after Bob's ping"
    );
    assert!(
        public_keys_after_bob_ping.contains(&bob_worker.public_key),
        "Pool should still contain Bob's public key after his ping"
    );

    // Verify that Alice cannot ping anymore (she's no longer registered)
    println!("Testing that Alice cannot ping after being replaced...");
    let result = ping_worker(&alice, &solver_registry).await?;
    assert!(
        !result.is_success(),
        "Alice should not be able to ping after being replaced"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    println!("Test passed: Worker can register after inactive worker timeout");

    Ok(())
}

#[tokio::test]
async fn test_worker_ping_without_registration() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for worker ping without registration...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, _owner, alice, _bob, mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Get pool account ID
    let pool_account_id = get_pool_account_id(&solver_registry, 0);
    println!("\n [LOG] Pool Account ID: {pool_account_id}");

    // Verify no public keys exist for the pool initially
    let initial_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert!(
        initial_public_keys.is_empty(),
        "Pool should have no public keys initially"
    );
    println!(
        "\n [LOG] Initial public keys count: {}",
        initial_public_keys.len()
    );

    // Try to ping without being registered as a worker
    println!("Attempting to ping without worker registration...");
    let result = ping_worker(&alice, &solver_registry).await?;

    // Ping should fail without registration
    assert!(
        !result.is_success(),
        "Ping should fail without worker registration"
    );

    let error = result.into_result().unwrap_err();
    println!("Expected error received: {error:?}");

    // Verify that the pool still has no public keys after failed ping attempt
    let final_public_keys = get_pool_public_keys(&mock_intents, &pool_account_id).await?;
    assert!(
        final_public_keys.is_empty(),
        "Pool should still have no public keys after failed ping attempt"
    );
    println!(
        "\n [LOG] Final public keys count: {}",
        final_public_keys.len()
    );

    println!("Test passed: Worker ping requires registration");

    Ok(())
}
