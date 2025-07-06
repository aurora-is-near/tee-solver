use near_gas::NearGas;
use near_sdk::{json_types::U128, NearToken};
use serde_json::json;

mod constants;
mod utils;

use constants::*;
use utils::*;

#[tokio::test]
async fn test_create_liquidity_pool() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test_create_liquidity_pool...");
    let sandbox = near_workspaces::sandbox().await?;

    // Deploy tokens
    let wnear = create_ft(
        &sandbox,
        "Wrapped NEAR",
        "wNEAR",
        24,
        NearToken::from_near(1_000_000_000).as_yoctonear(),
    )
    .await?;

    let usdc = create_ft(
        &sandbox,
        "USD Coin",
        "USDC",
        6,
        10_000_000_000_000_000,
    )
    .await?;

    let owner = create_account(&sandbox, "owner", 10).await?;
    let mock_intents = deploy_mock_intents(&sandbox).await?;
    let solver_registry = deploy_solver_registry(&sandbox, &mock_intents, &owner).await?;

    // Register contracts for tokens
    let _ = storage_deposit(&wnear, solver_registry.as_account()).await?;
    let _ = storage_deposit(&usdc, solver_registry.as_account()).await?;

    // Create liquidity pool
    let result = solver_registry
        .call("create_liquidity_pool")
        .args_json(json!({
            "token_ids": [wnear.id(), usdc.id()],
            "fee": 300
        }))
        .deposit(NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    
    assert!(result.is_success(), "Failed to create liquidity pool: {:?}", result.into_result().unwrap_err());

    // Verify pool was created
    let pool_info = solver_registry
        .view("get_pool_info")
        .args_json(json!({"pool_id": 0}))
        .await?;
    
    let pool: PoolInfo = serde_json::from_slice(&pool_info.result).unwrap();
    assert_eq!(pool.token_ids, vec![wnear.id(), usdc.id()]);
    assert_eq!(pool.fee, 300);
    assert_eq!(pool.shares_total_supply.0, 0);
    assert_eq!(pool.amounts.len(), 2);
    assert_eq!(pool.amounts[0].0, 0);
    assert_eq!(pool.amounts[1].0, 0);

    println!("✅ test_create_liquidity_pool passed");
    Ok(())
}

#[tokio::test]
async fn test_add_liquidity() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test_add_liquidity...");
    let sandbox = near_workspaces::sandbox().await?;

    // Deploy tokens
    let wnear = create_ft(
        &sandbox,
        "Wrapped NEAR",
        "wNEAR",
        24,
        NearToken::from_near(1_000_000_000).as_yoctonear(),
    )
    .await?;

    let usdc = create_ft(
        &sandbox,
        "USD Coin",
        "USDC",
        6,
        10_000_000_000_000_000,
    )
    .await?;

    let owner = create_account(&sandbox, "owner", 10).await?;
    let alice = create_account(&sandbox, "alice", 10).await?;
    let mock_intents = deploy_mock_intents(&sandbox).await?;
    let solver_registry = deploy_solver_registry(&sandbox, &mock_intents, &owner).await?;

    // Register accounts for tokens
    let _ = storage_deposit(&wnear, &alice).await?;
    let _ = storage_deposit(&usdc, &alice).await?;
    let _ = storage_deposit(&wnear, solver_registry.as_account()).await?;
    let _ = storage_deposit(&usdc, solver_registry.as_account()).await?;

    // Transfer tokens to Alice
    let _ = ft_transfer(
        &wnear,
        wnear.as_account(),
        &alice,
        NearToken::from_near(100).as_yoctonear(),
    )
    .await?;
    let _ = ft_transfer(&usdc, usdc.as_account(), &alice, 500_000_000).await?;

    // Create liquidity pool
    let result = solver_registry
        .call("create_liquidity_pool")
        .args_json(json!({
            "token_ids": [wnear.id(), usdc.id()],
            "fee": 300
        }))
        .deposit(NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(result.is_success());

    // Add liquidity: Transfer tokens first
    let wnear_amount = NearToken::from_near(10).as_yoctonear();
    let usdc_amount = 50_000_000u128;

    // Transfer wNEAR
    let result = alice
        .call(wnear.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": solver_registry.id(),
            "amount": wnear_amount.to_string(),
            "msg": json!({
                "AddLiquidity": {
                    "pool_id": 0,
                    "amounts": [wnear_amount.to_string(), usdc_amount.to_string()]
                }
            }).to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(result.is_success());

    // Transfer USDC
    let result = alice
        .call(usdc.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": solver_registry.id(),
            "amount": usdc_amount.to_string(),
            "msg": json!({
                "AddLiquidity": {
                    "pool_id": 0,
                    "amounts": [wnear_amount.to_string(), usdc_amount.to_string()]
                }
            }).to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(result.is_success());

    // Mint shares
    let result = solver_registry
        .call("add_liquidity")
        .args_json(json!({
            "pool_id": 0,
            "amounts": [U128(wnear_amount), U128(usdc_amount)]
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(result.is_success());

    // Verify shares were minted
    let shares = solver_registry
        .view("get_liquidity_provider_shares")
        .args_json(json!({
            "pool_id": 0,
            "account_id": alice.id()
        }))
        .await?;
    
    let shares_amount: U128 = serde_json::from_slice(&shares.result).unwrap();
    assert!(shares_amount.0 > 0, "Shares should be greater than 0");

    // Verify pool amounts were updated
    let pool_info = solver_registry
        .view("get_pool_info")
        .args_json(json!({"pool_id": 0}))
        .await?;
    
    let pool: PoolInfo = serde_json::from_slice(&pool_info.result).unwrap();
    assert_eq!(pool.amounts[0].0, wnear_amount);
    assert_eq!(pool.amounts[1].0, usdc_amount);
    assert_eq!(pool.shares_total_supply.0, shares_amount.0);

    println!("✅ test_add_liquidity passed");
    Ok(())
}

#[tokio::test]
async fn test_remove_liquidity() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test_remove_liquidity...");
    let sandbox = near_workspaces::sandbox().await?;

    // Deploy tokens
    let wnear = create_ft(
        &sandbox,
        "Wrapped NEAR",
        "wNEAR",
        24,
        NearToken::from_near(1_000_000_000).as_yoctonear(),
    )
    .await?;

    let usdc = create_ft(
        &sandbox,
        "USD Coin",
        "USDC",
        6,
        10_000_000_000_000_000,
    )
    .await?;

    let owner = create_account(&sandbox, "owner", 10).await?;
    let alice = create_account(&sandbox, "alice", 10).await?;
    let mock_intents = deploy_mock_intents(&sandbox).await?;
    let solver_registry = deploy_solver_registry(&sandbox, &mock_intents, &owner).await?;

    // Setup tokens and add initial liquidity
    setup_tokens_and_liquidity(&sandbox, &wnear, &usdc, &alice, &solver_registry).await?;

    // Get initial balances
    let initial_wnear = get_ft_balance(&wnear, &alice).await?;
    let initial_usdc = get_ft_balance(&usdc, &alice).await?;

    // Get current shares
    let shares = solver_registry
        .view("get_liquidity_provider_shares")
        .args_json(json!({
            "pool_id": 0,
            "account_id": alice.id()
        }))
        .await?;
    
    let shares_amount: U128 = serde_json::from_slice(&shares.result).unwrap();
    let shares_to_remove = shares_amount.0 / 2; // Remove half of shares

    // Remove liquidity
    let result = solver_registry
        .call("remove_liquidity")
        .args_json(json!({
            "pool_id": 0,
            "shares": U128(shares_to_remove)
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(result.is_success());

    // Verify tokens were returned
    let final_wnear = get_ft_balance(&wnear, &alice).await?;
    let final_usdc = get_ft_balance(&usdc, &alice).await?;

    assert!(final_wnear > initial_wnear, "wNEAR should be returned");
    assert!(final_usdc > initial_usdc, "USDC should be returned");

    // Verify shares were burned
    let remaining_shares = solver_registry
        .view("get_liquidity_provider_shares")
        .args_json(json!({
            "pool_id": 0,
            "account_id": alice.id()
        }))
        .await?;
    
    let remaining_shares_amount: U128 = serde_json::from_slice(&remaining_shares.result).unwrap();
    assert_eq!(remaining_shares_amount.0, shares_amount.0 - shares_to_remove);

    println!("✅ test_remove_liquidity passed");
    Ok(())
}

#[tokio::test]
async fn test_claim_rewards() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test_claim_rewards...");
    let sandbox = near_workspaces::sandbox().await?;

    // Deploy tokens
    let wnear = create_ft(
        &sandbox,
        "Wrapped NEAR",
        "wNEAR",
        24,
        NearToken::from_near(1_000_000_000).as_yoctonear(),
    )
    .await?;

    let usdc = create_ft(
        &sandbox,
        "USD Coin",
        "USDC",
        6,
        10_000_000_000_000_000,
    )
    .await?;

    let owner = create_account(&sandbox, "owner", 10).await?;
    let alice = create_account(&sandbox, "alice", 10).await?;
    let mock_intents = deploy_mock_intents(&sandbox).await?;
    let solver_registry = deploy_solver_registry(&sandbox, &mock_intents, &owner).await?;

    // Setup tokens and add initial liquidity
    setup_tokens_and_liquidity(&sandbox, &wnear, &usdc, &alice, &solver_registry).await?;

    // Get initial balances
    let initial_wnear = get_ft_balance(&wnear, &alice).await?;
    let initial_usdc = get_ft_balance(&usdc, &alice).await?;

    // Check pending rewards (should be 0 initially)
    let pending_rewards = solver_registry
        .view("get_pending_rewards")
        .args_json(json!({
            "pool_id": 0,
            "account_id": alice.id()
        }))
        .await?;
    
    let rewards: Vec<U128> = serde_json::from_slice(&pending_rewards.result).unwrap();
    assert_eq!(rewards[0].0, 0);
    assert_eq!(rewards[1].0, 0);

    // Try to claim rewards (should fail as no rewards available)
    let result = solver_registry
        .call("claim_rewards")
        .args_json(json!({
            "pool_id": 0
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    
    // This should fail as there are no rewards to claim
    assert!(!result.is_success());

    // Verify balances didn't change
    let final_wnear = get_ft_balance(&wnear, &alice).await?;
    let final_usdc = get_ft_balance(&usdc, &alice).await?;

    assert_eq!(final_wnear, initial_wnear);
    assert_eq!(final_usdc, initial_usdc);

    println!("✅ test_claim_rewards passed");
    Ok(())
}

#[tokio::test]
async fn test_multiple_liquidity_providers() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test_multiple_liquidity_providers...");
    let sandbox = near_workspaces::sandbox().await?;

    // Deploy tokens
    let wnear = create_ft(
        &sandbox,
        "Wrapped NEAR",
        "wNEAR",
        24,
        NearToken::from_near(1_000_000_000).as_yoctonear(),
    )
    .await?;

    let usdc = create_ft(
        &sandbox,
        "USD Coin",
        "USDC",
        6,
        10_000_000_000_000_000,
    )
    .await?;

    let owner = create_account(&sandbox, "owner", 10).await?;
    let alice = create_account(&sandbox, "alice", 10).await?;
    let bob = create_account(&sandbox, "bob", 10).await?;
    let mock_intents = deploy_mock_intents(&sandbox).await?;
    let solver_registry = deploy_solver_registry(&sandbox, &mock_intents, &owner).await?;

    // Setup tokens for both users
    setup_tokens_for_user(&sandbox, &wnear, &usdc, &alice).await?;
    setup_tokens_for_user(&sandbox, &wnear, &usdc, &bob).await?;

    // Register contracts for tokens
    let _ = storage_deposit(&wnear, solver_registry.as_account()).await?;
    let _ = storage_deposit(&usdc, solver_registry.as_account()).await?;

    // Create liquidity pool
    let result = solver_registry
        .call("create_liquidity_pool")
        .args_json(json!({
            "token_ids": [wnear.id(), usdc.id()],
            "fee": 300
        }))
        .deposit(NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(result.is_success());

    // Alice adds liquidity first
    let alice_wnear = NearToken::from_near(10).as_yoctonear();
    let alice_usdc = 50_000_000u128;
    add_liquidity_for_user(&solver_registry, &wnear, &usdc, &alice, 0, alice_wnear, alice_usdc).await?;

    // Bob adds liquidity second
    let bob_wnear = NearToken::from_near(20).as_yoctonear();
    let bob_usdc = 100_000_000u128;
    add_liquidity_for_user(&solver_registry, &wnear, &usdc, &bob, 0, bob_wnear, bob_usdc).await?;

    // Verify both users have shares
    let alice_shares = solver_registry
        .view("get_liquidity_provider_shares")
        .args_json(json!({
            "pool_id": 0,
            "account_id": alice.id()
        }))
        .await?;
    
    let bob_shares = solver_registry
        .view("get_liquidity_provider_shares")
        .args_json(json!({
            "pool_id": 0,
            "account_id": bob.id()
        }))
        .await?;
    
    let alice_shares_amount: U128 = serde_json::from_slice(&alice_shares.result).unwrap();
    let bob_shares_amount: U128 = serde_json::from_slice(&bob_shares.result).unwrap();

    assert!(alice_shares_amount.0 > 0, "Alice should have shares");
    assert!(bob_shares_amount.0 > 0, "Bob should have shares");

    // Verify pool total amounts
    let pool_info = solver_registry
        .view("get_pool_info")
        .args_json(json!({"pool_id": 0}))
        .await?;
    
    let pool: PoolInfo = serde_json::from_slice(&pool_info.result).unwrap();
    assert_eq!(pool.amounts[0].0, alice_wnear + bob_wnear);
    assert_eq!(pool.amounts[1].0, alice_usdc + bob_usdc);
    assert_eq!(pool.shares_total_supply.0, alice_shares_amount.0 + bob_shares_amount.0);

    println!("✅ test_multiple_liquidity_providers passed");
    Ok(())
}

#[tokio::test]
async fn test_invalid_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting test_invalid_operations...");
    let sandbox = near_workspaces::sandbox().await?;

    // Deploy tokens
    let wnear = create_ft(
        &sandbox,
        "Wrapped NEAR",
        "wNEAR",
        24,
        NearToken::from_near(1_000_000_000).as_yoctonear(),
    )
    .await?;

    let usdc = create_ft(
        &sandbox,
        "USD Coin",
        "USDC",
        6,
        10_000_000_000_000_000,
    )
    .await?;

    let owner = create_account(&sandbox, "owner", 10).await?;
    let alice = create_account(&sandbox, "alice", 10).await?;
    let mock_intents = deploy_mock_intents(&sandbox).await?;
    let solver_registry = deploy_solver_registry(&sandbox, &mock_intents, &owner).await?;

    // Test: Remove liquidity from non-existent pool
    let result = solver_registry
        .call("remove_liquidity")
        .args_json(json!({
            "pool_id": 999,
            "shares": U128(1000000000000000000000000)
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(!result.is_success(), "Should fail for non-existent pool");

    // Test: Claim rewards from non-existent pool
    let result = solver_registry
        .call("claim_rewards")
        .args_json(json!({
            "pool_id": 999
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(!result.is_success(), "Should fail for non-existent pool");

    // Test: Add liquidity without depositing tokens first
    let result = solver_registry
        .call("add_liquidity")
        .args_json(json!({
            "pool_id": 0,
            "amounts": [U128(1000000000000000000000000), U128(50000000)]
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(!result.is_success(), "Should fail without depositing tokens first");

    println!("✅ test_invalid_operations passed");
    Ok(())
}

// Helper functions

async fn setup_tokens_and_liquidity(
    sandbox: &near_workspaces::Worker<near_workspaces::network::Sandbox>,
    wnear: &near_workspaces::Contract,
    usdc: &near_workspaces::Contract,
    user: &near_workspaces::Account,
    solver_registry: &near_workspaces::Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    // Register accounts for tokens
    let _ = storage_deposit(wnear, user).await?;
    let _ = storage_deposit(usdc, user).await?;
    let _ = storage_deposit(wnear, solver_registry.as_account()).await?;
    let _ = storage_deposit(usdc, solver_registry.as_account()).await?;

    // Transfer tokens to user
    let _ = ft_transfer(
        wnear,
        wnear.as_account(),
        user,
        NearToken::from_near(100).as_yoctonear(),
    )
    .await?;
    let _ = ft_transfer(usdc, usdc.as_account(), user, 500_000_000).await?;

    // Create liquidity pool
    let result = solver_registry
        .call("create_liquidity_pool")
        .args_json(json!({
            "token_ids": [wnear.id(), usdc.id()],
            "fee": 300
        }))
        .deposit(NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(result.is_success());

    // Add initial liquidity
    let wnear_amount = NearToken::from_near(10).as_yoctonear();
    let usdc_amount = 50_000_000u128;
    add_liquidity_for_user(solver_registry, wnear, usdc, user, 0, wnear_amount, usdc_amount).await?;

    Ok(())
}

async fn setup_tokens_for_user(
    sandbox: &near_workspaces::Worker<near_workspaces::network::Sandbox>,
    wnear: &near_workspaces::Contract,
    usdc: &near_workspaces::Contract,
    user: &near_workspaces::Account,
) -> Result<(), Box<dyn std::error::Error>> {
    // Register accounts for tokens
    let _ = storage_deposit(wnear, user).await?;
    let _ = storage_deposit(usdc, user).await?;

    // Transfer tokens to user
    let _ = ft_transfer(
        wnear,
        wnear.as_account(),
        user,
        NearToken::from_near(100).as_yoctonear(),
    )
    .await?;
    let _ = ft_transfer(usdc, usdc.as_account(), user, 500_000_000).await?;

    Ok(())
}

async fn add_liquidity_for_user(
    solver_registry: &near_workspaces::Contract,
    wnear: &near_workspaces::Contract,
    usdc: &near_workspaces::Contract,
    user: &near_workspaces::Account,
    pool_id: u32,
    wnear_amount: u128,
    usdc_amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
    // Transfer wNEAR
    let result = user
        .call(wnear.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": solver_registry.id(),
            "amount": wnear_amount.to_string(),
            "msg": json!({
                "AddLiquidity": {
                    "pool_id": pool_id,
                    "amounts": [wnear_amount.to_string(), usdc_amount.to_string()]
                }
            }).to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(result.is_success());

    // Transfer USDC
    let result = user
        .call(usdc.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": solver_registry.id(),
            "amount": usdc_amount.to_string(),
            "msg": json!({
                "AddLiquidity": {
                    "pool_id": pool_id,
                    "amounts": [wnear_amount.to_string(), usdc_amount.to_string()]
                }
            }).to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(result.is_success());

    // Mint shares
    let result = solver_registry
        .call("add_liquidity")
        .args_json(json!({
            "pool_id": pool_id,
            "amounts": [U128(wnear_amount), U128(usdc_amount)]
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    assert!(result.is_success());

    Ok(())
}

async fn get_ft_balance(
    ft: &near_workspaces::Contract,
    account: &near_workspaces::Account,
) -> Result<u128, Box<dyn std::error::Error>> {
    let result = ft
        .view("ft_balance_of")
        .args_json(json!({
            "account_id": account.id()
        }))
        .await?;
    
    let balance: U128 = serde_json::from_slice(&result.result).unwrap();
    Ok(balance.0)
}
