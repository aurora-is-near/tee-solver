use near_sdk::NearToken;
use near_sdk::serde_json::json;
use tokio::join;

mod common;

use common::constants::*;
use common::utils::*;

#[tokio::test]
async fn test_create_liquidity_pool_parallel_failure() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test for parallel create_liquidity_pool failure...");
    let sandbox = near_workspaces::sandbox().await?;

    // Setup test environment
    let (wnear, usdc, _owner, _alice, _bob, _mock_intents, solver_registry) =
        setup_test_environment(&sandbox, DEFAULT_WORKER_PING_TIMEOUT_MS).await?;

    // Make two parallel calls to create_liquidity_pool
    // Both will try to create pool-0.{contract_id} because they both read pools.len() as 0
    println!("Making two parallel create_liquidity_pool calls...");

    let wnear_id = wnear.id().clone();
    let usdc_id = usdc.id().clone();
    let deposit = NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000); // 1.5 NEAR

    // Clone the contract reference and IDs for parallel access
    let solver_registry1 = solver_registry.clone();
    let solver_registry2 = solver_registry.clone();
    let wnear_id1 = wnear_id.clone();
    let usdc_id1 = usdc_id.clone();
    let wnear_id2 = wnear_id.clone();
    let usdc_id2 = usdc_id.clone();
    let deposit1 = deposit;
    let deposit2 = deposit;

    // Create two parallel tasks that both try to create pool with ID 0
    let call1 = async move {
        solver_registry1
            .call("create_liquidity_pool")
            .args_json(json!({
                "token_ids": [wnear_id1, usdc_id1],
                "fee": 300
            }))
            .deposit(deposit1)
            .max_gas()
            .transact()
            .await
    };

    let call2 = async move {
        solver_registry2
            .call("create_liquidity_pool")
            .args_json(json!({
                "token_ids": [wnear_id2, usdc_id2],
                "fee": 300
            }))
            .deposit(deposit2)
            .max_gas()
            .transact()
            .await
    };

    // Execute both calls in parallel
    let (result1, result2) = join!(call1, call2);

    let result1 = result1?;
    let result2 = result2?;

    // One should succeed, one should fail
    let success_count = u32::from(result1.is_success()) + u32::from(result2.is_success());
    let failure_count = u32::from(!result1.is_success()) + u32::from(!result2.is_success());

    println!("Success count: {success_count}, Failure count: {failure_count}");

    // At least one should fail
    assert!(
        failure_count >= 1,
        "At least one parallel create_liquidity_pool call should fail"
    );

    // Check that the failure is due to account creation failure
    if !result1.is_success() {
        let error = result1.into_result().unwrap_err();
        println!("First call error: {error:?}");
        // The error should contain "Failed to create liquidity pool account"
        let error_str = format!("{error:?}");
        assert!(
            error_str.contains("Failed to create liquidity pool account")
                || error_str.contains("Account already exists"),
            "Expected account creation failure, got: {error_str}"
        );
    }

    if !result2.is_success() {
        let error = result2.into_result().unwrap_err();
        println!("Second call error: {error:?}");
        // The error should contain "Failed to create liquidity pool account"
        let error_str = format!("{error:?}");
        assert!(
            error_str.contains("Failed to create liquidity pool account")
                || error_str.contains("Account already exists"),
            "Expected account creation failure, got: {error_str}"
        );
    }

    // Verify that exactly one pool was created
    let pool_len = get_pool_len(&solver_registry).await?;
    assert_eq!(
        pool_len, 1,
        "Exactly one pool should be created despite parallel calls"
    );

    println!(
        "Test passed: Parallel create_liquidity_pool calls properly handle account creation failure"
    );

    Ok(())
}
