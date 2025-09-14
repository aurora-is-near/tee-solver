use near_gas::NearGas;
use near_sdk::NearToken;
use serde_json::json;

mod common;

use common::constants::*;
use common::utils::*;

#[tokio::test]
async fn test_withdraw_assets_from_pool() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting withdraw assets test...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, _bob, mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

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
    let deposit_amount_wnear = NearToken::from_near(10).as_yoctonear();
    let deposit_amount_usdc = 50_000_000;

    let _ = deposit_into_pool(&solver_registry, &funder, 0, &wnear, deposit_amount_wnear).await?;
    let _ = deposit_into_pool(&solver_registry, &funder, 0, &usdc, deposit_amount_usdc).await?;

    // Get pool account ID
    let pool_account_id = get_pool_account_id(&solver_registry, 0);
    println!("Pool account ID: {}", pool_account_id);

    // Check balances in mock intents contract before withdrawal
    let token_ids = vec![wnear.id(), usdc.id()];
    let balances_before =
        get_mock_intents_balances(&mock_intents, &pool_account_id, token_ids.clone()).await?;
    println!(
        "Balances before withdrawal - wNEAR: {}, USDC: {}",
        balances_before[0], balances_before[1]
    );

    // Verify that the pool has the expected balances
    assert_eq!(
        balances_before[0], deposit_amount_wnear,
        "Pool should have correct wNEAR balance"
    );
    assert_eq!(
        balances_before[1], deposit_amount_usdc,
        "Pool should have correct USDC balance"
    );

    // Withdraw some wNEAR from the pool (simulating stopping the solver)
    let withdraw_amount_wnear = NearToken::from_near(5).as_yoctonear(); // Withdraw 5 NEAR
    println!("Withdrawing {} wNEAR from pool...", withdraw_amount_wnear);

    let withdraw_result = withdraw_from_pool(
        &solver_registry,
        &owner,
        0,
        &wnear.id(),
        withdraw_amount_wnear,
    )
    .await?;

    assert!(
        withdraw_result.is_success(),
        "Withdrawal should succeed: {:#?}",
        withdraw_result.into_result().unwrap_err()
    );

    // Check balances in mock intents contract after withdrawal
    let balances_after =
        get_mock_intents_balances(&mock_intents, &pool_account_id, token_ids.clone()).await?;
    println!(
        "Balances after withdrawal - wNEAR: {}, USDC: {}",
        balances_after[0], balances_after[1]
    );

    // Verify that the pool balance decreased by the withdrawal amount
    assert_eq!(
        balances_after[0],
        balances_before[0] - withdraw_amount_wnear,
        "Pool wNEAR balance should decrease by withdrawal amount"
    );
    assert_eq!(
        balances_after[1], balances_before[1],
        "Pool USDC balance should remain unchanged"
    );

    // Withdraw some USDC from the pool
    let withdraw_amount_usdc = 25_000_000; // Withdraw 25 USDC
    println!("Withdrawing {} USDC from pool...", withdraw_amount_usdc);

    let withdraw_result_usdc = withdraw_from_pool(
        &solver_registry,
        &owner,
        0,
        &usdc.id(),
        withdraw_amount_usdc,
    )
    .await?;

    assert!(
        withdraw_result_usdc.is_success(),
        "USDC withdrawal should succeed: {:#?}",
        withdraw_result_usdc.into_result().unwrap_err()
    );

    // Check final balances
    let balances_final =
        get_mock_intents_balances(&mock_intents, &pool_account_id, token_ids).await?;
    println!(
        "Final balances - wNEAR: {}, USDC: {}",
        balances_final[0], balances_final[1]
    );

    // Verify final balances
    assert_eq!(
        balances_final[0],
        deposit_amount_wnear - withdraw_amount_wnear,
        "Final wNEAR balance should be correct"
    );
    assert_eq!(
        balances_final[1],
        deposit_amount_usdc - withdraw_amount_usdc,
        "Final USDC balance should be correct"
    );

    println!("Test passed: Asset withdrawal from pool completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_withdraw_assets_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting withdraw assets error handling test...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, owner, alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, 10 * 60 * 1000).await?;

    // Create a liquidity pool
    create_liquidity_pool(&solver_registry, &wnear, &usdc).await?;

    // Approve compose hash
    approve_compose_hash(&owner, &solver_registry).await?;

    // Register worker (Alice)
    let result = register_worker_alice(&alice, &solver_registry, 0).await?;
    assert!(
        result.is_success(),
        "Worker registration should succeed: {:#?}",
        result.into_result().unwrap_err()
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

    // Deposit some tokens into liquidity pool
    let deposit_amount_wnear = NearToken::from_near(10).as_yoctonear();
    let deposit_amount_usdc = 50_000_000;

    let _ = deposit_into_pool(&solver_registry, &funder, 0, &wnear, deposit_amount_wnear).await?;
    let _ = deposit_into_pool(&solver_registry, &funder, 0, &usdc, deposit_amount_usdc).await?;

    // Test 1: Try to withdraw with invalid amount (0)
    println!("Testing withdrawal with invalid amount (0)...");
    let invalid_withdraw_result =
        withdraw_from_pool(&solver_registry, &owner, 0, &wnear.id(), 0).await?;

    assert!(
        !invalid_withdraw_result.is_success(),
        "Withdrawal with amount 0 should fail"
    );

    // Test 2: Try to withdraw with invalid token ID
    println!("Testing withdrawal with invalid token ID...");
    let invalid_token = create_account(&sandbox, "invalid-token", 10).await?;
    let invalid_token_withdraw_result = withdraw_from_pool(
        &solver_registry,
        &owner,
        0,
        &invalid_token.id(),
        NearToken::from_near(1).as_yoctonear(),
    )
    .await?;

    assert!(
        !invalid_token_withdraw_result.is_success(),
        "Withdrawal with invalid token ID should fail"
    );

    // Test 3: Try to withdraw from non-existent pool
    println!("Testing withdrawal from non-existent pool...");
    let invalid_pool_result = withdraw_from_pool(
        &solver_registry,
        &owner,
        999, // Non-existent pool ID
        &wnear.id(),
        NearToken::from_near(1).as_yoctonear(),
    )
    .await?;

    assert!(
        !invalid_pool_result.is_success(),
        "Withdrawal from non-existent pool should fail"
    );

    // Test 4: Try to withdraw as non-owner (should fail)
    println!("Testing withdrawal as non-owner...");
    let non_owner_withdraw_result = alice
        .call(solver_registry.id(), "withdraw_from_pool")
        .args_json(json!({
            "pool_id": 0,
            "token_id": wnear.id(),
            "amount": NearToken::from_near(1).as_yoctonear().to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;

    assert!(
        !non_owner_withdraw_result.is_success(),
        "Withdrawal as non-owner should fail"
    );

    println!("Test passed: Error handling for asset withdrawal completed successfully");
    Ok(())
}
